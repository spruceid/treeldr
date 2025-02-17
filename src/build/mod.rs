use crate::{
	function::FunctionRef,
	module::ItemPathBuf,
	syntax::{
		ast::{self, Ident},
		Span,
	},
	value::number::NumberParseError,
	Attribute, Attributes, Function, Module, ModuleRef, Package, PackageId, Registry, Value,
};
use std::hash::Hash;

mod scope;
use iref::{InvalidIri, IriBuf};
use ll1::Spanned;
use rdf_types::InterpretationMut;
pub use scope::*;

mod ty;
pub use ty::*;

mod function;
pub use function::*;

mod expr;
mod path;
mod pattern;

mod module;
pub use module::*;

mod typing;
pub use typing::*;

#[derive(Debug, thiserror::Error)]
pub enum Error<P> {
	#[error("invalid number: {1}")]
	Number(Span, NumberParseError),

	#[error("invalid string")]
	String(Span),

	#[error("invalid IRI")]
	Iri(Span, String),

	#[error("invalid language tag `{1}`")]
	Language(Span, String),

	#[error("unknown IRI prefix `{1}`")]
	UnknownPrefix(Span, String),

	#[error("`{1}` is not an IRI prefix")]
	NotAPrefix(Span, String),

	#[error("unknown variable `{0}`")]
	UnknownVariable(Ident),

	#[error("`{0}` is not a variable")]
	NotAVariable(Ident),

	#[error("unknown type `{1}`")]
	UnknownType(Span, ItemPathBuf),

	#[error("`{1}` is not a type")]
	NotAType(Span, ItemPathBuf),

	#[error("unknown function `{1}`")]
	UnknownFunction(Span, ItemPathBuf),

	#[error("`{1}` is not a function")]
	NotAFunction(Span, ItemPathBuf),

	#[error("unknown field")]
	UnknownField(Span),

	#[error("unknown variant `{0}`")]
	UnknownVariant(Ident),

	#[error("invalid number of arguments (expected {expected}, found {found})")]
	ArgumentCount {
		span: Span,
		expected: usize,
		found: usize,
	},

	#[error("type mismatch")]
	TypeMismatch(Span),

	#[error("expected RDF resource")]
	ExpectedResource(Span),

	#[error("expected exactly one binding")]
	ExpectedExactlyOneBinding(Span),

	#[error("expected const value")]
	ExpectedConstValue(Span),

	#[error("item `{0}` is already defined")]
	ItemRedefinition(Ident, BuildItem<P>, BuildItem<P>),
}

impl<P> Error<P> {
	pub fn span(&self) -> Span {
		match self {
			Self::Number(span, _) => *span,
			Self::String(span) => *span,
			Self::Iri(span, _) => *span,
			Self::Language(span, _) => *span,
			Self::UnknownPrefix(span, _) => *span,
			Self::NotAPrefix(span, _) => *span,
			Self::UnknownVariable(i) => i.span(),
			Self::NotAVariable(i) => i.span(),
			Self::UnknownType(span, _) => *span,
			Self::NotAType(span, _) => *span,
			Self::UnknownFunction(span, _) => *span,
			Self::NotAFunction(span, _) => *span,
			Self::UnknownField(span) => *span,
			Self::UnknownVariant(i) => i.span(),
			Self::ArgumentCount { span, .. } => *span,
			Self::TypeMismatch(span) => *span,
			Self::ExpectedResource(span) => *span,
			Self::ExpectedExactlyOneBinding(span) => *span,
			Self::ExpectedConstValue(span) => *span,
			Self::ItemRedefinition(id, _, _) => id.span(),
		}
	}
}

pub struct BuildContext<'a, E: ?Sized + Registry, D> {
	/// RDF context.
	rdf: &'a mut D,

	/// Registry.
	registry: &'a E,

	/// Type variables.
	type_variables: TypeVariables<E::PackageId, E::Resource>,
}

impl<'a, E: ?Sized + Registry, D> BuildContext<'a, E, D> {
	pub fn new(rdf: &'a mut D, registry: &'a E) -> Self {
		Self {
			rdf,
			registry,
			type_variables: TypeVariables::new(),
		}
	}
}

pub struct BuildingPackage<'ast, P> {
	/// Namespace index.
	id: P,

	/// Types.
	types: Vec<TypeDefinition<'ast, P>>,

	/// Functions.
	functions: Vec<FunctionDefinition<'ast, P>>,

	/// Modules.
	modules: Vec<Option<BuildModule<'ast, P>>>,
}

impl<'ast, P> BuildingPackage<'ast, P> {
	pub fn new(id: P) -> Self {
		Self {
			id,
			types: Vec::new(),
			functions: Vec::new(),
			modules: Vec::new(),
		}
	}

	pub fn get_module(&self, i: u32) -> Option<&BuildModule<'ast, P>> {
		self.modules.get(i as usize).and_then(Option::as_ref)
	}

	fn begin_module(
		&mut self,
		f: impl FnOnce(&mut Self, ModuleRef<P>) -> Result<BuildModule<'ast, P>, Error<P>>,
	) -> Result<ModuleRef<P>, Error<P>>
	where
		P: Copy,
	{
		let module_ref = ModuleRef::new(self.id, self.modules.len() as u32);
		self.modules.push(None);
		self.modules[module_ref.index as usize] = Some(f(self, module_ref)?);
		Ok(module_ref)
	}

	pub fn declare_type(
		&mut self,
		module: ModuleRef<P>,
		metadata: &'ast ast::Metadata,
		value: &'ast ast::TypeDefinition,
		variable: TypeVariableRef,
	) {
		self.types.push(TypeDefinition {
			module,
			metadata,
			value,
			variable,
		});
	}

	pub fn import_function_signature<E, D>(
		&self,
		context: &mut BuildContext<E, D>,
		r: FunctionRef<E::PackageId>,
	) -> Option<BuildSignature>
	where
		P: Copy + Ord + Hash,
		E: ?Sized + Registry<PackageId = P>,
		E::Resource: Clone,
	{
		if r.package == self.id {
			Some(self.functions[r.index as usize].signature.clone())
		} else {
			context.registry.get_function(r).map(|f| {
				BuildSignature::import(&f.signature, |&ty_ref| {
					context
						.type_variables
						.import_type_ref(context.registry, ty_ref)
				})
			})
		}
	}
}

impl<'ast, P: Copy + Ord + Hash> BuildingPackage<'ast, P> {
	/// Declare a function.
	pub fn declare_function(
		&mut self,
		module: ModuleRef<P>,
		metadata: &'ast ast::Metadata,
		value: &'ast ast::FunctionDefinition,
		signature: BuildSignature,
	) -> FunctionRef<P> {
		let f_ref = FunctionRef::new(self.id, self.functions.len() as u32);

		self.functions.push(FunctionDefinition {
			module,
			metadata,
			value,
			signature,
		});

		f_ref
	}

	pub fn build<E, D>(
		&self,
		context: &mut BuildContext<E, D>,
	) -> Result<ProtoPackage<E::PackageId, E::Resource>, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P>,
		E::Resource: Clone + Ord + Hash,
		D: InterpretationMut<Resource = E::Resource>,
	{
		for item in &self.types {
			item.build(context, self)?;
		}

		Ok(ProtoPackage {
			id: self.id,
			functions: self
				.functions
				.iter()
				.map(|item| item.build(context, self))
				.collect::<Result<_, _>>()?,
			modules: self
				.modules
				.iter()
				.map(|item| item.as_ref().unwrap().build(context, self))
				.collect::<Result<_, _>>()?,
		})
	}
}

pub struct ProtoPackage<P, R> {
	id: P,

	functions: Vec<Function<P, R, TypeVariableRef>>,

	modules: Vec<Module<P, R, TypeVariableRef>>,
}

impl<P, R> ProtoPackage<P, R>
where
	P: PackageId + Ord + Hash,
	R: Clone + Ord,
{
	pub fn resolve<E, D>(
		self,
		context: BuildContext<E, D>,
	) -> Result<Package<P, R>, TypeResolutionError<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
	{
		let mut package = Package::new(self.id);

		let map = context
			.type_variables
			.resolve(context.registry, &mut package)?;

		for f in self.functions {
			package.insert_function(f.map_types(|x| map[&x]));
		}

		for (i, m) in self.modules.into_iter().enumerate() {
			let m = m.map_types(|x| map[&x]);

			if i == 0 {
				package.set_root(m);
			} else {
				package.insert_module(m);
			}
		}

		Ok(package)
	}
}

pub trait Declare<'ast, P, R> {
	type Target;

	fn declare<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &mut BuildingPackage<'ast, P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
		D: InterpretationMut<Resource = R>;
}

pub trait Build<'ast, P, R> {
	type Target;

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
		D: InterpretationMut<Resource = R>;
}

impl ast::FullIri {
	fn decode<P>(&self) -> Result<IriBuf, Error<P>> {
		let len = self.as_str().len();
		let content = &self.as_str()[1..(len - 1)];
		IriBuf::new(content.to_owned()).map_err(|InvalidIri(i)| Error::Iri(self.span(), i))
	}
}

impl ast::DocComment {
	pub fn decode(&self) -> &str {
		&self.as_str()[2..]
	}
}

impl<'ast, P, R> Build<'ast, P, R> for ast::DocComment {
	type Target = Value<R>;

	fn build<E, D>(
		&'ast self,
		_context: &mut BuildContext<'_, E, D>,
		_namespace: &BuildingPackage<'ast, P>,
		_scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
		D: InterpretationMut<Resource = R>,
	{
		Ok(Value::string(self.decode().to_owned()))
	}
}

impl<'ast, P, R: Clone + Ord> Build<'ast, P, R> for ast::Metadata {
	type Target = Attributes<R>;

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
		D: InterpretationMut<Resource = R>,
	{
		let mut result = Attributes::new();

		for d in &self.doc {
			result.insert(
				Attribute::Documentation,
				d.build(context, namespace, scope)?,
			);
		}

		Ok(result)
	}
}
