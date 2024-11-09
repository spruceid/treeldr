use std::{
	borrow::Cow,
	collections::{BTreeMap, HashMap},
	hash::Hash,
};

use educe::Educe;
use iref::{InvalidIri, Iri, IriBuf};
use langtag::{InvalidLangTag, LangTagBuf};
use ll1::{Span, Spanned};
use rdf_types::{LiteralType, Quad, Term, RDF_FIRST, RDF_NIL, RDF_REST, XSD_STRING};

use crate::{
	expr::{self, Bound, Expr, InnerExpr, ListComponents},
	function::{self, FunctionRef, Signature},
	rdf::RdfContextMut,
	ty::{self, TypeDesc, Variant},
	value::{Number, NumberParseError, TypedMap, TypedValue, TypedValueDesc, TypedValueInnerDesc},
	Attribute, Attributes, DatasetPattern, Environment, Function, Item, ItemPath, ItemPathBuf,
	ItemRef, Literal, Module, Registry, TermPattern, Type, TypeRef, Value,
};

use super::ast::{self, Ident, ListDescription};

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

#[derive(Debug, Educe)]
#[educe(Clone, Copy)]
pub enum BuildItemRef<'a, 'ast, R> {
	Prefix(&'a Iri),
	Variable(u32),
	Type(TypeRef),
	TypeVariable(TypeVariableRef),
	Function(FunctionRef),
	BuildModule(&'a BuildModule<'ast>),
	Module(&'a Module<R>),
}

impl<'a, 'ast, R> BuildItemRef<'a, 'ast, R> {
	pub fn get(&self, path: &ItemPath) -> Option<Self> {
		if path.is_empty() {
			Some(*self)
		} else {
			match self {
				Self::BuildModule(m) => m.get(path),
				Self::Module(m) => m.get(path).map(Into::into),
				_ => None,
			}
		}
	}

	pub fn as_prefix(&self) -> Option<&'a str> {
		match self {
			Self::Prefix(p) => Some(p),
			_ => None,
		}
	}

	pub fn as_variable(&self) -> Option<u32> {
		match self {
			Self::Variable(x) => Some(*x),
			_ => None,
		}
	}

	pub fn import_type(
		&self,
		registry: &mut Registry<R>,
		type_variables: &mut TypeVariables<R>,
	) -> Option<TypeVariableRef> {
		match self {
			Self::Type(t) => Some(type_variables.import_type_ref(registry, *t)),
			Self::TypeVariable(t) => Some(*t),
			_ => None,
		}
	}

	pub fn as_function(&self) -> Option<FunctionRef> {
		match self {
			Self::Function(f) => Some(*f),
			_ => None,
		}
	}
}

impl<'a, 'ast, R> From<ItemRef<'a, R>> for BuildItemRef<'a, 'ast, R> {
	fn from(value: ItemRef<'a, R>) -> Self {
		match value {
			ItemRef::Type(t) => BuildItemRef::Type(t),
			ItemRef::Function(f) => BuildItemRef::Function(f),
			ItemRef::Module(m) => BuildItemRef::Module(m),
		}
	}
}

pub enum Scope<'a, R: Clone> {
	Root(&'a Environment<'a, R>),
	Module(&'a Self, &'a BuildModule<'a>),
	SubScope(SubScope<'a, R>),
}

impl<'a, R: Clone> Scope<'a, R> {
	pub fn environment(&self) -> &'a Environment<'a, R> {
		match self {
			Self::Root(e) => e,
			Self::Module(scope, _) => scope.environment(),
			Self::SubScope(scope) => scope.parent.environment(),
		}
	}

	pub fn offset(&self) -> u32 {
		match self {
			Self::Root(_) => 0,
			Self::Module(parent, _) => parent.offset(),
			Self::SubScope(s) => s.offset(),
		}
	}

	pub fn len(&self) -> u32 {
		match self {
			Self::Root(_) => 0,
			Self::Module(parent, _) => parent.len(),
			Self::SubScope(s) => s.len(),
		}
	}

	pub fn variable(&self, i: u32) -> Option<(&str, TypeVariableRef)> {
		match self {
			Self::Root(_) => None,
			Self::Module(parent, _) => parent.variable(i),
			Self::SubScope(s) => s.variable(i),
		}
	}

	pub fn get_self(&self) -> Option<BuildItemRef<R>> {
		match self {
			Self::Root(_) => None,
			Self::Module(_, m) => Some(BuildItemRef::BuildModule(m)),
			Self::SubScope(s) => s.get_self(),
		}
	}

	pub fn get(&self, path: &ItemPath) -> Option<BuildItemRef<R>> {
		match path.split_first() {
			Some((first, rest)) => self
				.get_by_name(first)
				.and_then(|item| item.get(ItemPath::from_slice(rest))),
			None => self.get_self(),
		}
	}

	pub fn get_by_name(&self, name: &str) -> Option<BuildItemRef<R>> {
		match self {
			Self::Root(env) => env.get_by_name(name).map(Into::into),
			Self::Module(parent, module) => module
				.get_by_name(name)
				.or_else(|| parent.get_by_name(name)),
			Self::SubScope(s) => s.get_by_name(name),
		}
	}

	pub fn type_of(&self, i: u32) -> Option<TypeVariableRef> {
		self.variable(i).map(|(_, ty)| ty)
	}

	pub fn begin<D>(&self, context: &mut BuildContext<R, D>, bindings: &ast::Bindings) -> Scope<R> {
		let mut sub_scope = SubScope::new(self);

		for ident in &bindings.variables {
			let ty = context.type_variables.fresh(Type::resource());
			sub_scope.insert(ident, ty);
		}

		Scope::SubScope(sub_scope)
	}
}

pub struct SubScope<'a, R: Clone> {
	parent: &'a Scope<'a, R>,
	variables: Vec<(String, TypeVariableRef)>,
	bindings: HashMap<String, u32>,
	len: u32,
}

impl<'a, R: Clone> SubScope<'a, R> {
	pub fn new(parent: &'a Scope<'a, R>) -> Self {
		Self {
			parent,
			variables: Vec::new(),
			bindings: HashMap::new(),
			len: parent.len(),
		}
	}

	pub fn offset(&self) -> u32 {
		self.parent.len()
	}

	pub fn len(&self) -> u32 {
		self.len
	}

	pub fn variable(&self, i: u32) -> Option<(&str, TypeVariableRef)> {
		let offset = self.parent.len();

		if i < offset {
			self.parent.variable(i)
		} else {
			self.variables
				.get((i - offset) as usize)
				.map(|(n, t)| (n.as_str(), *t))
		}
	}

	pub fn get_self(&self) -> Option<BuildItemRef<R>> {
		self.parent.get_self()
	}

	pub fn get(&self, path: &ItemPath) -> Option<BuildItemRef<R>> {
		match path.split_first() {
			Some((first, rest)) => self
				.get_by_name(first)
				.and_then(|item| item.get(ItemPath::from_slice(rest))),
			None => self.get_self(),
		}
	}

	pub fn get_by_name(&self, name: &str) -> Option<BuildItemRef<R>> {
		match self.bindings.get(name) {
			Some(i) => Some(BuildItemRef::Variable(*i)),
			None => self.parent.get_by_name(name),
		}
	}

	pub fn insert(&mut self, ident: &ast::Ident, ty: TypeVariableRef) -> u32 {
		let i = self.len;
		self.len += 1;
		self.variables.push((ident.as_str().to_owned(), ty));
		self.bindings.insert(ident.as_str().to_owned(), i);
		i
	}
}

pub struct BuildContext<'a, R, D> {
	/// RDF context.
	rdf: &'a mut D,

	/// Registry.
	registry: &'a mut Registry<R>,

	/// Namespace index.
	namespace: u32,

	/// Type resolver.
	type_variables: TypeVariables<R>,

	/// Functions.
	functions: Vec<FunctionDefinition<R>>,
}

impl<'a, R, D> BuildContext<'a, R, D> {
	pub fn new(rdf: &'a mut D, registry: &'a mut Registry<R>) -> Self {
		let namespace = registry.create_namespace();

		Self {
			rdf,
			registry,
			namespace,
			type_variables: TypeVariables::new(),
			functions: Vec::new(),
		}
	}

	fn declare_function(&mut self, signature: Signature<R, TypeVariableRef>) -> FunctionRef
	where
		R: Clone,
	{
		let f_ref = FunctionRef::new(self.namespace, self.functions.len() as u32);

		self.functions.push(FunctionDefinition {
			signature,
			value: None,
		});

		f_ref
	}

	fn define_function(
		&mut self,
		r: FunctionRef,
		attributes: Attributes<R>,
		body: function::Body<R, TypeVariableRef>,
	) where
		R: Clone,
	{
		let f = &mut self.functions[r.index as usize];
		f.value = Some(Function {
			attributes,
			signature: f.signature.clone(),
			body,
		});
	}

	fn import_function_signature(&mut self, r: FunctionRef) -> Option<Signature<R, TypeVariableRef>>
	where
		R: Clone,
	{
		if r.namespace == self.namespace {
			Some(self.functions[r.index as usize].signature.clone())
		} else {
			self.registry.get_function(r).map(|f| {
				f.signature.mapped_types(|&ty_ref| {
					self.type_variables.import_type_ref(self.registry, ty_ref)
				})
			})
		}
	}

	pub fn resolve(self) -> Result<HashMap<TypeVariableRef, TypeRef>, TypeResolutionError>
	where
		R: Clone + Ord,
	{
		let map = self.type_variables.resolve(self.registry, self.namespace)?;

		for f in self.functions {
			let value = f.value.unwrap();
			self.registry.insert_function(
				self.namespace,
				Function {
					attributes: value.attributes,
					signature: value.signature.map_types(|x| map[&x]),
					body: value.body.map_types(|x| map[&x]),
				},
			);
		}

		Ok(map)
	}
}

struct FunctionDefinition<R> {
	signature: Signature<R, TypeVariableRef>,
	value: Option<Function<R, TypeVariableRef>>,
}

#[derive(Debug)]
enum BuildItem<'ast> {
	Prefix(IriBuf),
	Type(
		TypeVariableRef,
		&'ast ast::TypeDefinition,
		&'ast ast::Metadata,
	),
	Function(
		FunctionRef,
		&'ast ast::FunctionDefinition,
		&'ast ast::Metadata,
	),
	Module(BuildModule<'ast>, &'ast ast::Metadata),
}

impl<'ast> BuildItem<'ast> {
	fn as_ref<R>(&self) -> BuildItemRef<'_, 'ast, R> {
		match self {
			Self::Prefix(p) => BuildItemRef::Prefix(p),
			Self::Type(r, _, _) => BuildItemRef::TypeVariable(*r),
			Self::Function(r, _, _) => BuildItemRef::Function(*r),
			Self::Module(m, _) => BuildItemRef::BuildModule(m),
		}
	}
}

#[derive(Debug, Default)]
pub struct BuildModule<'ast> {
	/// Items.
	items: HashMap<String, BuildItem<'ast>>,
}

impl<'ast> BuildModule<'ast> {
	pub fn get<R>(&self, path: &ItemPath) -> Option<BuildItemRef<'_, 'ast, R>> {
		match path.split_first() {
			Some((first, rest)) => self
				.get_by_name(first)
				.and_then(|item| item.get(ItemPath::from_slice(rest))),
			None => Some(BuildItemRef::BuildModule(self)),
		}
	}

	fn get_by_name<R>(&self, name: &str) -> Option<BuildItemRef<'_, 'ast, R>> {
		self.items.get(name).map(BuildItem::as_ref)
	}
}

pub trait Build<'ast, R: Clone> {
	type Target;

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>;
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::Module {
	type Target = BuildModule<'ast>;

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		// Declare prefixes.
		let mut prefixed_module = BuildModule::default();
		for item in &self.items {
			if let ast::Item::Prefix(ast) = &item.value {
				let iri = ast.value.build(scope)?;
				prefixed_module
					.items
					.insert(ast.ident.as_str().to_owned(), BuildItem::Prefix(iri));
			}
		}

		// Declare.
		let scope = Scope::Module(scope, &prefixed_module);
		let mut result = BuildModule::default();
		for item in &self.items {
			match &item.value {
				ast::Item::Prefix(p_ast) => {
					let iri = p_ast.value.build(&scope)?;
					result
						.items
						.insert(p_ast.ident.as_str().to_owned(), BuildItem::Prefix(iri));
				}
				ast::Item::Type(t_ast) => {
					let x = context.type_variables.fresh(TypeValue::Undefined);
					if let Some(_) = result.items.insert(
						t_ast.ident.as_str().to_owned(),
						BuildItem::Type(x, t_ast, &item.metadata),
					) {
						todo!() // type redefinition.
					}
				}
				ast::Item::Function(f_ast) => {
					let mut sub_scope = SubScope::new(&scope);

					let args = f_ast
						.args
						.iter()
						.map(|a| {
							let type_ref = context.type_variables.fresh(TypeValue::Undefined);
							sub_scope.insert(a.ident(), type_ref);
							type_ref
						})
						.collect();

					let sub_scope = Scope::SubScope(sub_scope);

					let signature = Signature {
						args,
						return_type: context.type_variables.fresh(TypeValue::Undefined),
						dataset: f_ast
							.where_bound
							.as_ref()
							.map(|b| b.dataset.build(context, &sub_scope))
							.transpose()?
							.unwrap_or_default(),
					};

					let f_ref = context.declare_function(signature);
					if let Some(_) = result.items.insert(
						f_ast.ident.as_str().to_owned(),
						BuildItem::Function(f_ref, f_ast, &item.metadata),
					) {
						todo!() // function redefinition
					}
				}
				ast::Item::Module(m) => {
					let m_scope = Scope::Root(scope.environment());
					if let Some(_) = result.items.insert(
						m.ident.as_str().to_owned(),
						BuildItem::Module(m.value.build(context, &m_scope)?, &item.metadata),
					) {
						todo!() // module redefinition
					}
				}
			}
		}

		result.items.extend(prefixed_module.items);

		Ok(result)
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for BuildModule<'ast> {
	type Target = Module<R, TypeVariableRef>;

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let mut result = Module::default();

		for (name, item) in &self.items {
			match item {
				BuildItem::Prefix(_) => (),
				BuildItem::Type(x, ast, metadata) => {
					let attributes = metadata.build(context, scope)?;
					let y = ast
						.value
						.build_with_attributes(context, scope, attributes)?;
					context.type_variables.assert_eq(*x, y);
					result.insert(name, Item::Type(*x));
				}
				BuildItem::Function(f_ref, ast, metadata) => {
					let attributes = metadata.build(context, scope)?;
					let mut sub_scope = SubScope::new(scope);

					let args: Vec<_> = ast
						.args
						.iter()
						.map(|a| {
							Ok(match a {
								ast::ArgDef::Typed(_, ident, ty, _) => {
									let type_ref = ty.expr.build(context, scope)?;
									sub_scope.insert(ident, type_ref);
									type_ref
								}
								ast::ArgDef::Untyped(ident) => {
									let type_ref = context.type_variables.fresh(Type::resource());
									sub_scope.insert(ident, type_ref.clone());
									type_ref
								}
							})
						})
						.collect::<Result<_, _>>()?;

					let body = ast.body.build(context, &Scope::SubScope(sub_scope))?;

					let return_type = match &ast.type_ {
						Some(ty) => ty.expr.build(context, scope)?,
						None => body.type_,
					};

					let signature = context.import_function_signature(*f_ref).unwrap().clone();
					for (&a, &b) in args.iter().zip(&signature.args) {
						context.type_variables.assert_eq(a, b);
					}

					context
						.type_variables
						.assert_eq(return_type, signature.return_type);

					context.define_function(
						*f_ref,
						attributes,
						function::Body::Expr(Box::new(body)),
					);
					result.insert(name, Item::Function(*f_ref));
				}
				BuildItem::Module(m, metadata) => {
					let mut attributes = metadata.build(context, scope)?;
					let m_scope = Scope::Root(scope.environment());
					let mut compiled_m = m.build(context, &m_scope)?;
					std::mem::swap(&mut attributes, &mut compiled_m.attributes);
					compiled_m.attributes.extend(attributes);
					result.insert(name, Item::Module(compiled_m));
				}
			}
		}

		Ok(result)
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::ItemPath {
	type Target = ItemPathBuf;

	fn build<D>(
		&'ast self,
		_context: &mut BuildContext<'_, R, D>,
		_scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let mut result = Vec::new();
		self.build_into(&mut result);
		Ok(ItemPathBuf(result))
	}
}

impl ast::ItemPath {
	fn build_into(&self, result: &mut Vec<String>) {
		result.push(self.first.as_str().to_owned());
		if let Some(rest) = &self.rest {
			rest.rest.build_into(result);
		}
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::TypeExpr {
	type Target = TypeVariableRef;

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		self.build_with_attributes(context, scope, Attributes::new())
	}
}

impl ast::DocComment {
	pub fn decode(&self) -> &str {
		&self.as_str()[2..]
	}
}

impl<'ast, R: Clone + Ord> Build<'ast, R> for ast::DocComment {
	type Target = Value<R>;

	fn build<D>(
		&'ast self,
		_context: &mut BuildContext<'_, R, D>,
		_scope: &Scope<R>,
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
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let mut result = Attributes::new();

		for d in &self.doc {
			result.insert(Attribute::Documentation, d.build(context, scope)?);
		}

		Ok(result)
	}
}

impl ast::TypeExpr {
	fn build_with_attributes<R, D>(
		&self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
		attributes: Attributes<R>,
	) -> Result<TypeVariableRef, Error>
	where
		R: Clone + Ord + Hash,
		D: RdfContextMut<R>,
	{
		match self {
			Self::Path(path) => {
				let path = path.build(context, scope)?;

				scope
					.get(&path)
					.ok_or_else(|| Error::UnknownType(path.clone()))?
					.import_type(context.registry, &mut context.type_variables)
					.ok_or_else(|| Error::NotAType(path.clone()))
			}
			Self::Literal(expr) => {
				let (_, ty) = expr.build(context, scope)?;
				Ok(context
					.type_variables
					.fresh(Type::new(attributes, TypeDesc::Literal(ty))))
			}
			Self::List(expr) => {
				let ty = expr.build(context, scope)?;
				Ok(context
					.type_variables
					.fresh(Type::new(attributes, TypeDesc::List(ty))))
			}
			Self::Map(expr) => {
				let ty = expr.build(context, scope)?;
				Ok(context
					.type_variables
					.fresh(Type::new(attributes, TypeDesc::Struct(ty))))
			}
			Self::Enum(expr) => {
				let ty = expr.build(context, scope)?;
				Ok(context
					.type_variables
					.fresh(Type::new(attributes, TypeDesc::Enum(ty))))
			}
		}
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::ListTypeExpr {
	type Target = ty::List<TypeVariableRef>;

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		Ok(ty::List {
			prefix: self
				.prefix
				.iter()
				.map(|e| e.build(context, scope))
				.collect::<Result<_, _>>()?,
			rest: self
				.rest
				.as_ref()
				.map(|r| r.type_.build(context, scope))
				.transpose()?,
		})
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::MapTypeExpr {
	type Target = ty::Struct<R, TypeVariableRef>;

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		Ok(ty::Struct {
			fields: self
				.entries
				.iter()
				.map(|f| {
					Ok((
						f.key.build(context, scope)?,
						ty::Field {
							attributes: f.metadata.build(context, scope)?,
							type_: f.value.build(context, scope)?,
							required: self.optional.is_none(),
						},
					))
				})
				.collect::<Result<_, _>>()?,
		})
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::EnumTypeExpr {
	type Target = ty::Enum<R, TypeVariableRef>;

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		Ok(ty::Enum {
			variants: self
				.variants
				.iter()
				.map(|v| {
					let variant = Variant {
						attributes: v.metadata.build(context, scope)?,
						type_: v.type_.build(context, scope)?,
					};

					Ok((v.ident.as_str().to_owned(), variant))
				})
				.collect::<Result<_, _>>()?,
		})
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::Expr {
	type Target = Expr<R, TypeVariableRef>;

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let (bound, inner, type_) = match self {
			Self::Let(_, bindings, _, inner) => {
				let (bound, sub_scope) = bindings.build(context, scope)?;
				let (inner, type_) = inner.build(context, &sub_scope)?;
				(bound, inner, type_)
			}
			Self::Inner(inner) => {
				let (inner, type_) = inner.build(context, scope)?;
				(expr::Bound::default(), inner, type_)
			}
		};

		Ok(Expr {
			type_,
			bound,
			inner,
		})
	}
}

impl ast::Bindings {
	fn build<'ast, 'a, R, D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &'a Scope<R>,
	) -> Result<(expr::Bound<R, TypeVariableRef>, Scope<'a, R>), Error>
	where
		R: Clone + Ord + Hash,
		D: RdfContextMut<R>,
	{
		let mut intro = Vec::with_capacity(self.variables.len());
		let mut sub_scope = SubScope::new(scope);

		for ident in &self.variables {
			let ty_ref = context.type_variables.fresh(Type::resource());
			intro.push(ty_ref);
			sub_scope.insert(ident, ty_ref);
		}

		let sub_scope = Scope::SubScope(sub_scope);

		let dataset = match &self.bound {
			Some(b) => b.dataset.build(context, &sub_scope)?,
			None => DatasetPattern::new(),
		};

		Ok((expr::Bound { intro, dataset }, sub_scope))
	}
}

impl<'ast, R: Clone + Ord> Build<'ast, R> for ast::RdfDataset {
	type Target = DatasetPattern<R>;

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let mut result = DatasetPattern::new();

		for statement in &self.statements {
			statement.build(context, scope, &mut result)?;
		}

		Ok(result)
	}
}

impl ast::RdfStatement {
	fn build<R, D>(
		&self,
		context: &mut BuildContext<R, D>,
		scope: &Scope<R>,
		result: &mut DatasetPattern<R>,
	) -> Result<(), Error>
	where
		R: Clone + Ord,
		D: RdfContextMut<R>,
	{
		let subject = self.subject.build(context, scope)?;

		for po in &self.predicates_objects {
			po.build(context, scope, result, &subject)?;
		}

		Ok(())
	}
}

impl ast::RdfPredicateObjects {
	fn build<R, D>(
		&self,
		context: &mut BuildContext<R, D>,
		scope: &Scope<R>,
		result: &mut DatasetPattern<R>,
		subject: &TermPattern<R>,
	) -> Result<(), Error>
	where
		R: Clone + Ord,
		D: RdfContextMut<R>,
	{
		let predicate = self.predicate.build(context, scope)?;

		for o in &self.objects {
			let object = o.build(context, scope)?;
			result.insert(Quad(subject.clone(), predicate.clone(), object, None));
		}

		Ok(())
	}
}

impl ast::RdfTerm {
	fn build_unscoped(&self) -> Result<Term, Error> {
		match self {
			Self::Iri(i) => i.build_unscoped().map(Term::iri),
			Self::Literal(l) => l.build_unscoped().map(Term::Literal),
		}
	}

	fn build<R, D>(&self, context: &mut BuildContext<R, D>, scope: &Scope<R>) -> Result<R, Error>
	where
		R: Clone,
		D: RdfContextMut<R>,
	{
		match self {
			Self::Iri(i) => {
				let iri = i.build(scope)?;
				Ok(context.rdf.insert_iri(Cow::Owned(iri)))
			}
			Self::Literal(l) => l.build(context.rdf, scope),
		}
	}
}

impl ast::RdfPattern {
	fn build<R, D>(
		&self,
		context: &mut BuildContext<R, D>,
		scope: &Scope<R>,
	) -> Result<TermPattern<R>, Error>
	where
		R: Clone,
		D: RdfContextMut<R>,
	{
		match self {
			Self::Var(ident) => scope
				.get_by_name(ident.as_str())
				.ok_or_else(|| Error::UnknownVariable(ident.clone()))?
				.as_variable()
				.ok_or_else(|| Error::NotAVariable(ident.clone()))
				.map(TermPattern::Var),
			Self::Iri(i) => {
				let iri = i.build(scope)?;
				Ok(TermPattern::Resource(
					context.rdf.insert_iri(Cow::Owned(iri)),
				))
			}
			Self::Literal(l) => l.build(context.rdf, scope).map(TermPattern::Resource),
		}
	}
}

impl ast::RdfLiteral {
	fn build_unscoped(&self) -> Result<rdf_types::Literal, Error> {
		let value = self.value.decode()?;
		match &self.annotation {
			None => Ok(rdf_types::Literal::new(
				value,
				LiteralType::Any(XSD_STRING.to_owned()),
			)),
			Some(ast::RdfLiteralAnnotation::Type(_, iri)) => {
				let iri = iri.build_unscoped()?;
				Ok(rdf_types::Literal::new(value, LiteralType::Any(iri)))
			}
			Some(ast::RdfLiteralAnnotation::Language(lang)) => {
				let tag = lang.decode()?;
				Ok(rdf_types::Literal::new(value, LiteralType::LangString(tag)))
			}
		}
	}

	fn build<R: Clone>(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		scope: &Scope<R>,
	) -> Result<R, Error> {
		let value = self.value.decode()?;
		match &self.annotation {
			None => Ok(rdf.insert_literal(value, Cow::Borrowed(XSD_STRING))),
			Some(ast::RdfLiteralAnnotation::Type(_, iri)) => {
				let iri = iri.build(scope)?;
				Ok(rdf.insert_literal(value, Cow::Owned(iri)))
			}
			Some(ast::RdfLiteralAnnotation::Language(lang)) => {
				let tag = lang.decode()?;
				Ok(rdf.insert_lang_string(value, tag))
			}
		}
	}
}

impl ast::LangTag {
	fn decode(&self) -> Result<LangTagBuf, Error> {
		LangTagBuf::new(self.as_str()[1..].to_owned())
			.map_err(|InvalidLangTag(t)| Error::Language(self.span(), t))
	}
}

impl ast::Iri {
	fn build_unscoped(&self) -> Result<IriBuf, Error> {
		match self {
			Self::Full(iri) => iri.decode(),
			Self::Compact(compact_iri) => {
				let prefix = compact_iri.as_str().split_once(':').unwrap().0;
				Err(Error::UnknownPrefix(compact_iri.span(), prefix.to_owned()))
			}
		}
	}

	fn build<R: Clone>(&self, scope: &Scope<R>) -> Result<IriBuf, Error> {
		match self {
			Self::Full(iri) => iri.decode(),
			Self::Compact(compact_iri) => compact_iri.build(scope),
		}
	}
}

impl ast::FullIri {
	fn decode(&self) -> Result<IriBuf, Error> {
		let len = self.as_str().len();
		let content = &self.as_str()[1..(len - 1)];
		IriBuf::new(content.to_owned()).map_err(|InvalidIri(i)| Error::Iri(self.span(), i))
	}
}

impl ast::CompactIri {
	fn build<R: Clone>(&self, scope: &Scope<R>) -> Result<IriBuf, Error> {
		let (prefix_name, suffix) = self.as_str().split_once(':').unwrap();
		let prefix = scope
			.get_by_name(prefix_name)
			.ok_or_else(|| Error::UnknownPrefix(self.span(), prefix_name.to_owned()))?
			.as_prefix()
			.ok_or_else(|| Error::NotAPrefix(self.span(), prefix_name.to_owned()))?;
		IriBuf::new(format!("{prefix}{suffix}")).map_err(|InvalidIri(i)| Error::Iri(self.span(), i))
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::InnerExpr {
	type Target = (InnerExpr<R, TypeVariableRef>, TypeVariableRef);

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		match self {
			Self::Var(ident) => {
				let x = scope
					.get_by_name(ident.as_str())
					.ok_or_else(|| Error::UnknownVariable(ident.clone()))?
					.as_variable()
					.ok_or_else(|| Error::NotAVariable(ident.clone()))?;
				let type_ = scope.type_of(x).unwrap();
				Ok((InnerExpr::Var(x), type_))
			}
			Self::Literal(expr) => {
				let (e, ty) = expr.build(context, scope)?;
				let ty = context
					.type_variables
					.fresh(Type::new(Attributes::new(), TypeDesc::Literal(ty)));
				Ok((InnerExpr::Literal(e), ty))
			}
			Self::List(expr) => {
				let (e, ty) = expr.build(context, scope)?;
				let ty = context
					.type_variables
					.fresh(Type::new(Attributes::new(), TypeDesc::List(ty)));
				Ok((InnerExpr::List(e), ty))
			}
			Self::Map(expr) => {
				let (e, ty) = expr.build(context, scope)?;
				let ty = context
					.type_variables
					.fresh(Type::new(Attributes::new(), TypeDesc::Struct(ty)));
				Ok((InnerExpr::Map(e), ty))
			}
			Self::Match(expr) => {
				let (e, ty) = expr.build(context, scope)?;
				let ty = context
					.type_variables
					.fresh(Type::new(Attributes::new(), TypeDesc::Enum(ty)));
				Ok((InnerExpr::Match(e), ty))
			}
			Self::Call(expr) => {
				let (e, ty) = expr.build(context, scope)?;
				Ok((InnerExpr::Call(e), ty))
			}
		}
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::ListExpr {
	type Target = (expr::List<R, TypeVariableRef>, ty::List<TypeVariableRef>);

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		match &self.desc {
			ListDescription::Given(list) => {
				let mut items = Vec::with_capacity(list.len());
				let mut types = Vec::with_capacity(list.len());

				for item in list {
					let e = item.build(context, scope)?;
					types.push(e.type_);
					items.push(e);
				}

				Ok((
					expr::List::Explicit(expr::ExplicitList::new(items)),
					ty::List::from_prefix(types),
				))
			}
			ListDescription::All(_, bindings, mapping) => {
				let (bound, sub_scope) = bindings.build(context, scope)?;

				match bound.intro.as_slice() {
					&[item_ty] => {
						let body = Box::new(match mapping {
							Some(m) => m.expr.build(context, &sub_scope)?,
							None => Expr {
								type_: item_ty,
								bound: Bound::default(),
								inner: InnerExpr::Var(sub_scope.offset()),
							},
						});

						context.type_variables.assert_eq(item_ty, body.type_);

						let e = expr::List::Implicit(expr::ImplicitList::Unordered(
							expr::UnorderedList { bound, body },
						));

						Ok((e, ty::List::uniform(item_ty)))
					}
					_ => Err(Error::ExpectedExactlyOneBinding(self.span())),
				}
			}
			ListDescription::List(_, bindings, _, head, mapping) => {
				let (bound, sub_scope) = bindings.build(context, scope)?;

				match bound.intro.as_slice() {
					&[item_ty] => {
						let body = Box::new(match mapping {
							Some(m) => m.expr.build(context, &sub_scope)?,
							None => Expr {
								type_: item_ty,
								bound: Bound::default(),
								inner: InnerExpr::Var(sub_scope.offset()),
							},
						});

						context.type_variables.assert_eq(item_ty, body.type_);

						let e =
							expr::List::Implicit(expr::ImplicitList::Ordered(expr::OrderedList {
								components: ListComponents {
									first: context.rdf.insert_iri(Cow::Borrowed(RDF_FIRST)),
									rest: context.rdf.insert_iri(Cow::Borrowed(RDF_REST)),
									nil: context.rdf.insert_iri(Cow::Borrowed(RDF_NIL)),
								},
								head: Box::new(head.build(context, scope)?),
								bound,
								body,
							}));

						Ok((e, ty::List::uniform(item_ty)))
					}
					_ => Err(Error::ExpectedExactlyOneBinding(self.span())),
				}
			}
		}
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::MapExpr {
	type Target = (
		expr::Map<R, TypeVariableRef>,
		ty::Struct<R, TypeVariableRef>,
	);

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let mut fields = BTreeMap::new();
		let mut entries = TypedMap::new();

		for entry in &self.entries {
			let key = entry.key.build(context, scope)?;
			let value = entry.value.build(context, scope)?;

			fields.insert(
				key.type_,
				ty::Field {
					attributes: entry.metadata.build(context, scope)?,
					type_: value.type_,
					required: entry.optional.is_none(),
				},
			);

			entries.insert(key, value);
		}

		let e = expr::Map { entries };

		let t = ty::Struct { fields };

		Ok((e, t))
	}
}

impl ast::ConstValue {
	pub fn build_unscoped(&self) -> Result<Value, Error> {
		match self {
			Self::Resource(term) => term.build_unscoped().map(Value::Resource),
			Self::Literal(expr) => expr.build_unscoped().map(Value::Literal),
			Self::List(expr) => expr.build_unscoped().map(Value::List),
			Self::Map(expr) => expr.build_unscoped().map(Value::Map),
		}
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::ConstValue {
	type Target = TypedValue<R, TypeVariableRef>;

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let (inner, type_) = match self {
			Self::Resource(term) => {
				let term = term.build(context, scope)?;
				let ty = context.type_variables.fresh(Type::new(
					Attributes::new(),
					TypeDesc::Resource(ty::Resource::singleton(term.clone())),
				));
				(TypedValueInnerDesc::Resource(term), ty)
			}
			Self::Literal(expr) => {
				let (e, ty) = expr.build(context, scope)?;
				let ty = context
					.type_variables
					.fresh(Type::new(Attributes::new(), TypeDesc::Literal(ty)));
				(TypedValueInnerDesc::Literal(e), ty)
			}
			Self::List(expr) => {
				let (e, ty) = expr.build(context, scope)?;
				let ty = context
					.type_variables
					.fresh(Type::new(Attributes::new(), TypeDesc::List(ty)));
				(TypedValueInnerDesc::List(e), ty)
			}
			Self::Map(expr) => {
				let (e, ty) = expr.build(context, scope)?;
				let ty = context
					.type_variables
					.fresh(Type::new(Attributes::new(), TypeDesc::Struct(ty)));
				(TypedValueInnerDesc::Map(e), ty)
			}
		};

		Ok(TypedValue {
			type_,
			desc: TypedValueDesc::Constant(inner),
		})
	}
}

impl ast::ConstList {
	pub fn build_unscoped(&self) -> Result<Vec<Value>, Error> {
		self.items
			.iter()
			.map(|v| v.build_unscoped())
			.collect::<Result<_, _>>()
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::ConstList {
	type Target = (
		Vec<TypedValue<R, TypeVariableRef>>,
		ty::List<TypeVariableRef>,
	);

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let mut items = Vec::with_capacity(self.items.len());
		let mut types = Vec::with_capacity(self.items.len());

		for item in &self.items {
			let e = item.build(context, scope)?;
			types.push(e.type_);
			items.push(e);
		}

		Ok((items, ty::List::from_prefix(types)))
	}
}

impl ast::ConstMap {
	pub fn build_unscoped(&self) -> Result<BTreeMap<Value, Value>, Error> {
		self.entries
			.iter()
			.map(|entry| Ok((entry.key.build_unscoped()?, entry.value.build_unscoped()?)))
			.collect::<Result<_, _>>()
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::ConstMap {
	type Target = (
		TypedMap<R, TypedValue<R, TypeVariableRef>, TypeVariableRef>,
		ty::Struct<R, TypeVariableRef>,
	);

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let mut fields = BTreeMap::new();
		let mut entries = TypedMap::new();

		for entry in &self.entries {
			let key = entry.key.build(context, scope)?;
			let value = entry.value.build(context, scope)?;

			fields.insert(
				key.type_,
				ty::Field {
					attributes: Attributes::new(),
					type_: value.type_,
					required: true,
				},
			);

			entries.insert(key, value);
		}

		let t = ty::Struct { fields };

		Ok((entries, t))
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::MatchExpr {
	type Target = (
		expr::Match<R, TypeVariableRef>,
		ty::Enum<R, TypeVariableRef>,
	);

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let mut t = ty::Enum {
			variants: BTreeMap::new(),
		};

		let mut e = expr::Match {
			cases: BTreeMap::new(),
			order: Vec::with_capacity(self.cases.len()),
		};

		for case in &self.cases {
			let value = case.value.build(context, scope)?;
			let variant = Variant {
				attributes: case.metadata.build(context, scope)?,
				type_: value.type_,
			};
			t.variants.insert(case.name.as_str().to_owned(), variant);
			e.cases.insert(case.name.as_str().to_owned(), value);
			e.order.push(case.name.as_str().to_owned());
		}

		Ok((e, t))
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::CallExpr {
	type Target = (expr::Call<R, TypeVariableRef>, TypeVariableRef);

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let path = self.function.build(context, scope)?;

		let f_ref = scope
			.get(&path)
			.ok_or_else(|| Error::UnknownFunction(path.clone()))?
			.as_function()
			.ok_or_else(|| Error::NotAFunction(path.clone()))?;

		let signature = context.import_function_signature(f_ref).unwrap().clone();

		if self.args.len() != signature.args.len() {
			return Err(Error::ArgumentCount {
				expected: signature.args.len(),
				found: self.args.len(),
			});
		}

		let args = self
			.args
			.iter()
			.zip(&signature.args)
			.map(|(a, &a_ty)| {
				let e = a.build(context, scope)?;
				context.type_variables.assert_subtype_of(e.type_, a_ty);
				Ok(e)
			})
			.collect::<Result<_, _>>()?;

		let e = expr::Call {
			function: f_ref,
			args,
		};

		let t = context.type_variables.fresh(TypeValue::Undefined);
		context.type_variables.assert_eq(signature.return_type, t);

		Ok((e, t))
	}
}

impl ast::LiteralExpr {
	pub fn build_unscoped(&self) -> Result<Literal, Error> {
		match self {
			Self::Unit(_) => Ok(Literal::Unit),
			Self::True(_) => Ok(Literal::Boolean(true)),
			Self::False(_) => Ok(Literal::Boolean(false)),
			Self::Number(a) => a.build_unscoped().map(Literal::Number),
			Self::TextString(a) => a.decode().map(Literal::TextString),
		}
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::LiteralExpr {
	type Target = (Literal, ty::LiteralType);

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		scope: &Scope<R>,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		match self {
			Self::Unit(_) => Ok((Literal::Unit, ty::LiteralType::Unit)),
			Self::True(a) => {
				let (v, t) = a.build(context, scope)?;
				Ok((Literal::Boolean(v), ty::LiteralType::Boolean(t)))
			}
			Self::False(a) => {
				let (v, t) = a.build(context, scope)?;
				Ok((Literal::Boolean(v), ty::LiteralType::Boolean(t)))
			}
			Self::Number(a) => {
				let (v, t) = a.build(context, scope)?;
				Ok((Literal::Number(v), ty::LiteralType::Number(t)))
			}
			Self::TextString(a) => {
				let (v, t) = a.build(context, scope)?;
				Ok((Literal::TextString(v), ty::LiteralType::TextString(t)))
			}
		}
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::True {
	type Target = (bool, ty::Boolean);

	fn build<D>(
		&'ast self,
		_context: &mut BuildContext<'_, R, D>,
		_scope: &Scope<R>,
	) -> Result<Self::Target, Error> {
		Ok((true, ty::Boolean::singleton(true)))
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::False {
	type Target = (bool, ty::Boolean);

	fn build<D>(
		&'ast self,
		_context: &mut BuildContext<'_, R, D>,
		_scope: &Scope<R>,
	) -> Result<Self::Target, Error> {
		Ok((false, ty::Boolean::singleton(false)))
	}
}

impl ast::Number {
	pub fn build_unscoped(&self) -> Result<Number, Error> {
		self.as_str()
			.parse()
			.map_err(|e| Error::Number(self.span(), e))
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::Number {
	type Target = (Number, ty::Number);

	fn build<D>(
		&'ast self,
		_context: &mut BuildContext<'_, R, D>,
		_scope: &Scope<R>,
	) -> Result<Self::Target, Error> {
		let value = self.build_unscoped()?;
		Ok((value.clone(), ty::Number::singleton(value)))
	}
}

impl ast::TextString {
	pub fn decode(&self) -> Result<String, Error> {
		decode_string_literal(self.as_str(), self.span())
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::TextString {
	type Target = (String, ty::TextString);

	fn build<D>(
		&'ast self,
		_context: &mut BuildContext<'_, R, D>,
		_scope: &Scope<R>,
	) -> Result<Self::Target, Error> {
		let value = self.decode()?;
		Ok((value.clone(), ty::TextString::singleton(value)))
	}
}

impl ast::RdfString {
	pub fn decode(&self) -> Result<String, Error> {
		decode_string_literal(self.as_str(), self.span())
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::RdfString {
	type Target = (String, ty::TextString);

	fn build<D>(
		&'ast self,
		_context: &mut BuildContext<'_, R, D>,
		_scope: &Scope<R>,
	) -> Result<Self::Target, Error> {
		let value = self.decode()?;
		Ok((value.clone(), ty::TextString::singleton(value)))
	}
}

fn decode_string_literal(s: &str, span: Span) -> Result<String, Error> {
	let content = &s[1..(s.len() - 1)];
	let mut chars = content.chars();
	let mut value = String::with_capacity(content.len());

	while let Some(c) = chars.next() {
		let decoded_c = match c {
			'\\' => match chars.next() {
				Some('x') => {
					let a = chars
						.next()
						.and_then(|c| c.to_digit(16))
						.ok_or_else(|| Error::String(span))?;

					let b = chars
						.next()
						.and_then(|c| c.to_digit(16))
						.ok_or_else(|| Error::String(span))?;

					char::from_u32(a << 4 | b).ok_or_else(|| Error::String(span))?
				}
				Some('u' | 'U') => match chars.next() {
					Some('{') => {
						let mut value = 0;
						let mut i = 0;

						loop {
							match chars.next() {
								Some('}') if i >= 2 => break,
								Some(d) if i < 6 => {
									value = value << 4
										| d.to_digit(16).ok_or_else(|| Error::String(span))?;
									i += 1
								}
								_ => return Err(Error::String(span)),
							}
						}

						char::from_u32(value).ok_or_else(|| Error::String(span))?
					}
					_ => return Err(Error::String(span)),
				},
				Some('t') => '\t',
				Some('r') => '\r',
				Some('n') => '\n',
				Some('\'') => '\'',
				Some('"') => '"',
				Some('\\') => '\\',
				_ => return Err(Error::String(span)),
			},
			c => c,
		};

		value.push(decoded_c);
	}

	Ok(value)
}
