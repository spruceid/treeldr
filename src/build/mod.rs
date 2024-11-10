use crate::{
	function::FunctionRef,
	rdf::RdfContextMut,
	syntax::{
		ast::{self, Ident},
		Span,
	},
	value::NumberParseError,
	Attribute, Attributes, Environment, Function, ItemPathBuf, Module, ModuleRef, Registry, Value,
};
use std::hash::Hash;

mod scope;
use iref::{InvalidIri, IriBuf};
use ll1::Spanned;
pub use scope::*;

mod ty;
pub use ty::*;

mod function;
pub use function::*;

mod expr;
mod path;

mod module;
pub use module::*;

mod typing;
pub use typing::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
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

	#[error("unknown type `{0}`")]
	UnknownType(ItemPathBuf),

	#[error("`{0}` is not a type")]
	NotAType(ItemPathBuf),

	#[error("unknown function `{0}`")]
	UnknownFunction(ItemPathBuf),

	#[error("`{0}` is not a function")]
	NotAFunction(ItemPathBuf),

	#[error("unknown field")]
	UnknownField(Span),

	#[error("unknown variant `{0}`")]
	UnknownVariant(Ident),

	#[error("invalid number of arguments (expected {expected}, found {found})")]
	ArgumentCount { expected: usize, found: usize },

	#[error("type mismatch")]
	TypeMismatch(Span),

	#[error("expected RDF resource")]
	ExpectedResource(Span),

	#[error("expected exactly one binding")]
	ExpectedExactlyOneBinding(Span),
}

pub struct BuildContext<'a, R, D> {
	/// RDF context.
	rdf: &'a mut D,

	/// Registry.
	registry: &'a mut Registry<R>,

	/// Environment.
	environment: &'a Environment,

	/// Type variables.
	type_variables: TypeVariables<R>,
}

impl<'a, R, D> BuildContext<'a, R, D> {
	pub fn new(rdf: &'a mut D, registry: &'a mut Registry<R>, env: &'a Environment) -> Self {
		Self {
			rdf,
			registry,
			environment: env,
			type_variables: TypeVariables::new(),
		}
	}
}

pub struct BuildingNamespace<'ast> {
	/// Namespace index.
	id: u32,

	/// Types.
	types: Vec<TypeDefinition<'ast>>,

	/// Functions.
	functions: Vec<FunctionDefinition<'ast>>,

	/// Modules.
	modules: Vec<Option<BuildModule<'ast>>>,
}

impl<'ast> BuildingNamespace<'ast> {
	pub fn new(id: u32) -> Self {
		Self {
			id,
			types: Vec::new(),
			functions: Vec::new(),
			modules: Vec::new(),
		}
	}

	pub fn get_module(&self, i: u32) -> Option<&BuildModule<'ast>> {
		self.modules.get(i as usize).map(Option::as_ref).flatten()
	}

	fn begin_module(
		&mut self,
		f: impl FnOnce(&mut Self, ModuleRef) -> Result<BuildModule<'ast>, Error>,
	) -> Result<ModuleRef, Error> {
		let module_ref = ModuleRef::new(self.id, self.modules.len() as u32);
		self.modules.push(None);
		self.modules[module_ref.index as usize] = Some(f(self, module_ref)?);
		Ok(module_ref)
	}

	pub fn declare_type(
		&mut self,
		module: ModuleRef,
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

	/// Declare a function.
	pub fn declare_function(
		&mut self,
		module: ModuleRef,
		metadata: &'ast ast::Metadata,
		value: &'ast ast::FunctionDefinition,
		signature: BuildSignature,
	) -> FunctionRef {
		let f_ref = FunctionRef::new(self.id, self.functions.len() as u32);

		self.functions.push(FunctionDefinition {
			module,
			metadata,
			value,
			signature,
		});

		f_ref
	}

	pub fn import_function_signature<R, D>(
		&self,
		context: &mut BuildContext<R, D>,
		r: FunctionRef,
	) -> Option<BuildSignature>
	where
		R: Clone,
	{
		if r.namespace == self.id {
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

	pub fn build<R, D>(
		&self,
		context: &mut BuildContext<R, D>,
	) -> Result<UnresolvedNamespace<R>, Error>
	where
		R: Clone + Ord + Hash,
		D: RdfContextMut<R>,
	{
		for item in &self.types {
			item.build(context, self)?;
		}

		Ok(UnresolvedNamespace {
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

pub struct UnresolvedNamespace<R> {
	id: u32,

	functions: Vec<Function<R, TypeVariableRef>>,

	modules: Vec<Module<R, TypeVariableRef>>,
}

impl<R> UnresolvedNamespace<R> {
	pub fn resolve<D>(self, context: BuildContext<R, D>) -> Result<(), TypeResolutionError>
	where
		R: Clone + Ord,
	{
		let map = context.type_variables.resolve(context.registry, self.id)?;

		for f in self.functions {
			context
				.registry
				.insert_function(self.id, f.map_types(|x| map[&x]));
		}

		for m in self.modules {
			context
				.registry
				.insert_module(self.id, m.map_types(|x| map[&x]));
		}

		Ok(())
	}
}

pub trait Declare<'ast, R> {
	type Target;

	fn declare<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		namespace: &mut BuildingNamespace<'ast>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>;
}

pub trait Build<'ast, R> {
	type Target;

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>;
}

impl ast::FullIri {
	fn decode(&self) -> Result<IriBuf, Error> {
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

impl<'ast, R> Build<'ast, R> for ast::DocComment {
	type Target = Value<R>;

	fn build<D>(
		&'ast self,
		_context: &mut BuildContext<'_, R, D>,
		_namespace: &BuildingNamespace<'ast>,
		_scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		Ok(Value::string(self.decode().to_owned()))
	}
}

impl<'ast, R: Clone + Ord> Build<'ast, R> for ast::Metadata {
	type Target = Attributes<R>;

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
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
