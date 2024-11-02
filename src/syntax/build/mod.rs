use std::{
	borrow::Cow,
	collections::{BTreeMap, HashMap},
	hash::Hash,
};

use iref::{InvalidIri, IriBuf};
use langtag::{InvalidLangTag, LangTagBuf};
use ll1::{Span, Spanned};
use rdf_types::{Quad, RDF_FIRST, RDF_NIL, RDF_REST, XSD_STRING};

use crate::{
	eval::RdfContextMut, expr::{self, Bound, Expr, InnerExpr, ListComponents}, function::{self, FunctionRef, Signature}, ty::{self, TypeUnificationError, TypeUnifier}, value::{Number, NumberParseError, TypedMap, TypedValue, TypedValueDesc, TypedValueInnerDesc}, DatasetPattern, Function, Item, Literal, Module, Registry, RegistryReservation, TermPattern, Type, TypeRef
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

	#[error("unknown item `{0}`")]
	UnknownItem(Ident),

	#[error("`{0}` is not a variable")]
	NotAVariable(Ident),

	#[error("`{0}` is not a type")]
	NotAType(Ident),

	#[error("`{0}` is not a function")]
	NotAFunction(Ident),

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

pub enum ItemRef {
	Variable(u32),
	Type(TypeVariableRef),
	Function(FunctionRef),
}

impl ItemRef {
	pub fn as_variable(&self) -> Option<u32> {
		match self {
			Self::Variable(x) => Some(*x),
			_ => None,
		}
	}

	pub fn as_type(&self) -> Option<TypeVariableRef> {
		match self {
			Self::Type(t) => Some(*t),
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

pub enum Scope<'a> {
	Root(Option<&'a BuildModule>),
	SubScope(SubScope<'a>),
}

impl<'a> Scope<'a> {
	pub fn offset(&self) -> u32 {
		match self {
			Self::Root(_) => 0,
			Self::SubScope(s) => s.offset(),
		}
	}

	pub fn len(&self) -> u32 {
		match self {
			Self::Root(_) => 0,
			Self::SubScope(s) => s.len(),
		}
	}

	pub fn variable(&self, i: u32) -> Option<(&str, TypeVariableRef)> {
		match self {
			Self::Root(_) => None,
			Self::SubScope(s) => s.variable(i),
		}
	}

	pub fn get_prefix(&self, name: &str) -> Option<&str> {
		todo!()
	}

	pub fn get(&self, ident: &ast::Ident) -> Result<ItemRef, Error> {
		match self {
			Self::Root(_) => Err(Error::UnknownItem(ident.clone())),
			Self::SubScope(s) => s.get(ident),
		}
	}

	pub fn type_of(&self, i: u32) -> Option<TypeVariableRef> {
		self.variable(i).map(|(_, ty)| ty)
	}

	pub fn begin<R, C>(&self, context: &mut BuildContext<R, C>, bindings: &ast::Bindings) -> Scope {
		let mut sub_scope = SubScope::new(self);

		for ident in &bindings.variables {
			let ty = context.type_variables.fresh(Type::resource());
			sub_scope.insert(ident, ty);
		}

		Scope::SubScope(sub_scope)
	}
}

pub struct SubScope<'a> {
	parent: &'a Scope<'a>,
	variables: Vec<(String, TypeVariableRef)>,
	bindings: HashMap<String, u32>,
	len: u32,
}

impl<'a> SubScope<'a> {
	pub fn new(parent: &'a Scope<'a>) -> Self {
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

	pub fn get(&self, ident: &ast::Ident) -> Result<ItemRef, Error> {
		match self.bindings.get(ident.as_str()) {
			Some(i) => Ok(ItemRef::Variable(*i)),
			None => self.parent.get(ident),
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

pub struct BuildContext<'registry, 'context, 'ast, R, C> {
	/// Registry.
	registry: &'registry mut Registry<R>,

	/// Parent module.
	parent: &'context C,

	/// Type resolver.
	type_variables: TypeVariables<R>,

	/// Functions.
	functions: Vec<FunctionDefinition<'ast, R>>,
}

struct FunctionDefinition<'ast, R> {
	signature: Signature<TypeVariableRef>,
	ast: &'ast ast::Function,
	value: Option<Function<R, TypeVariableRef>>,
}

enum BuildItem {
	Type(TypeVariableRef),
	Function(FunctionRef),
	Module(BuildModule),
}

#[derive(Default)]
pub struct BuildModule {
	/// IRI prefixes.
	prefixes: HashMap<String, IriBuf>,

	/// Items.
	items: HashMap<String, BuildItem>,
}

impl BuildModule {
	fn get_prefix(&self, name: &str) -> Option<&str> {
		todo!()
	}
}

impl<'r, 'c, 'a, R, C> BuildContext<'r, 'c, 'a, R, C> {
	fn new(registry: &'r mut Registry<R>, parent: &'c C) -> Self {
		Self {
			registry,
			parent,
			type_variables: TypeVariables::new(),
			functions: Vec::new(),
		}
	}

	fn declare_function(
		&mut self,
		ast: &'a ast::Function,
		signature: Signature<TypeVariableRef>,
	) -> Result<FunctionRef, Error> {
		todo!()
	}

	fn define_function(&mut self, r: FunctionRef, body: function::Body<R, TypeVariableRef>) {
		todo!()
	}

	fn get_function(&self, r: FunctionRef) -> Option<&FunctionDefinition<'a, R>> {
		todo!()
	}
}

pub trait Build<'ast, R> {
	type Target;

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error>;
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::Module {
	type Target = BuildModule;

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		let mut result = BuildModule::default();

		// Declare.
		for item in &self.items {
			match item {
				ast::Item::Prefix(_) => {
					// nothing to do.
				}
				ast::Item::Function(f_ast) => {
					let signature = Signature {
						args: f_ast
							.args
							.iter()
							.map(|_| context.type_variables.fresh(TypeValue::Undefined))
							.collect(),
						return_type: context.type_variables.fresh(TypeValue::Undefined),
					};

					let f_ref = context.declare_function(f_ast, signature)?;
					if let Some(_) = result
						.items
						.insert(f_ast.ident.as_str().to_owned(), BuildItem::Function(f_ref))
					{
						todo!() // function redefinition
					}
				}
			}
		}

		Ok(result)
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for BuildModule {
	type Target = Module<TypeVariableRef>;

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		let mut result = Module::default();

		for (name, item) in &self.items {
			match item {
				BuildItem::Type(_) => {
					todo!()
				}
				BuildItem::Function(f_ref) => {
					let f_ast = context.get_function(*f_ref).unwrap().ast;
					let mut sub_scope = SubScope::new(scope);

					let args: Vec<_> = f_ast
						.args
						.iter()
						.map(|a| {
							Ok(match a {
								ast::ArgDef::Typed(_, ident, ty, _) => {
									let type_ref = ty.expr.build(rdf, context, scope)?;
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

					let body = f_ast
						.body
						.build(rdf, context, &Scope::SubScope(sub_scope))?;

					let return_type = match &f_ast.type_ {
						Some(ty) => ty.expr.build(rdf, context, scope)?,
						None => body.type_,
					};

					let signature = context.get_function(*f_ref).unwrap().signature.clone();
					for (&a, &b) in args.iter().zip(&signature.args) {
						context.type_variables.assert_eq(a, b);
					}

					context
						.type_variables
						.assert_eq(return_type, signature.return_type);

					context.define_function(*f_ref, function::Body::Expr(Box::new(body)));
					result.insert(name, Item::Function(*f_ref));
				}
				BuildItem::Module(_) => {
					todo!()
				}
			}
		}

		Ok(result)
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::TypeExpr {
	type Target = TypeVariableRef;

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		match self {
			Self::Var(ident) => scope
				.get(ident)?
				.as_type()
				.ok_or_else(|| Error::NotAType(ident.clone())),
			Self::Literal(expr) => {
				let (_, ty) = expr.build(rdf, context, scope)?;
				Ok(context.type_variables.fresh(Type::Literal(ty)))
			}
			Self::List(expr) => {
				let ty = expr.build(rdf, context, scope)?;
				Ok(context.type_variables.fresh(Type::List(ty)))
			}
			Self::Map(expr) => {
				let ty = expr.build(rdf, context, scope)?;
				Ok(context.type_variables.fresh(Type::Struct(ty)))
			}
			Self::Enum(expr) => {
				let ty = expr.build(rdf, context, scope)?;
				Ok(context.type_variables.fresh(Type::Enum(ty)))
			}
		}
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::ListTypeExpr {
	type Target = ty::List<TypeVariableRef>;

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		Ok(ty::List {
			prefix: self
				.prefix
				.iter()
				.map(|e| e.build(rdf, context, scope))
				.collect::<Result<_, _>>()?,
			rest: self
				.rest
				.as_ref()
				.map(|r| r.type_.build(rdf, context, scope))
				.transpose()?,
		})
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::MapTypeExpr {
	type Target = ty::Struct<TypeVariableRef>;

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		Ok(ty::Struct {
			fields: self
				.entries
				.iter()
				.map(|f| {
					Ok((
						f.key.build(rdf, context, scope)?,
						ty::Field {
							type_: f.value.build(rdf, context, scope)?,
							required: false,
						},
					))
				})
				.collect::<Result<_, _>>()?,
		})
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::EnumTypeExpr {
	type Target = ty::Enum<TypeVariableRef>;

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		Ok(ty::Enum {
			variants: self
				.variants
				.iter()
				.map(|v| {
					Ok((
						v.ident.as_str().to_owned(),
						v.type_.build(rdf, context, scope)?,
					))
				})
				.collect::<Result<_, _>>()?,
		})
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::Expr {
	type Target = Expr<R, TypeVariableRef>;

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		let (bound, inner, type_) = match self {
			Self::Let(_, bindings, _, inner) => {
				let (bound, sub_scope) = bindings.build(rdf, context, scope)?;
				let (inner, type_) = inner.build(rdf, context, &sub_scope)?;
				(bound, inner, type_)
			}
			Self::Inner(inner) => {
				let (inner, type_) = inner.build(rdf, context, scope)?;
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
	fn build<'ast, 'a, R: Clone + Ord + Hash, C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &'a Scope,
	) -> Result<(expr::Bound<R, TypeVariableRef>, Scope<'a>), Error> {
		let mut intro = Vec::with_capacity(self.variables.len());
		let dataset = DatasetPattern::new();
		let mut sub_scope = SubScope::new(scope);

		for ident in &self.variables {
			let ty_ref = context.type_variables.fresh(Type::resource());
			intro.push(ty_ref);
			sub_scope.insert(ident, ty_ref);
		}

		let sub_scope = Scope::SubScope(sub_scope);

		if let Some(b) = &self.bound {
			b.dataset.build(rdf, context, &sub_scope)?;
		}

		Ok((expr::Bound { intro, dataset }, sub_scope))
	}
}

impl<'ast, R: Clone + Ord> Build<'ast, R> for ast::RdfDataset {
	type Target = DatasetPattern<R>;

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		let mut result = DatasetPattern::new();

		for statement in &self.statements {
			statement.build(rdf, context, scope, &mut result)?;
		}

		Ok(result)
	}
}

impl ast::RdfStatement {
	fn build<R: Clone + Ord, C>(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<R, C>,
		scope: &Scope,
		result: &mut DatasetPattern<R>,
	) -> Result<(), Error> {
		let subject = self.subject.build(rdf, context, scope)?;

		for po in &self.predicates_objects {
			po.build(rdf, context, scope, result, &subject)?;
		}

		Ok(())
	}
}

impl ast::RdfPredicateObjects {
	fn build<R: Clone + Ord, C>(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<R, C>,
		scope: &Scope,
		result: &mut DatasetPattern<R>,
		subject: &TermPattern<R>,
	) -> Result<(), Error> {
		let predicate = self.predicate.build(rdf, context, scope)?;

		for o in &self.objects {
			let object = o.build(rdf, context, scope)?;
			result.insert(Quad(subject.clone(), predicate.clone(), object, None));
		}

		Ok(())
	}
}

impl ast::RdfTerm {
	fn build<R, C>(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		_context: &mut BuildContext<R, C>,
		scope: &Scope,
	) -> Result<TermPattern<R>, Error> {
		match self {
			Self::Var(ident) => scope
				.get(ident)?
				.as_variable()
				.ok_or_else(|| Error::NotAVariable(ident.clone()))
				.map(TermPattern::Var),
			Self::Iri(i) => {
				let iri = i.build(scope)?;
				Ok(TermPattern::Resource(rdf.insert_iri(Cow::Owned(iri))))
			}
			Self::Literal(l) => l.build(rdf, scope).map(TermPattern::Resource),
		}
	}
}

impl ast::RdfLiteral {
	fn build<R>(&self, rdf: &mut impl RdfContextMut<R>, scope: &Scope) -> Result<R, Error> {
		let value = self.value.decode()?;
		match &self.annotation {
			None => Ok(rdf.insert_literal(value, Cow::Borrowed(XSD_STRING))),
			Some(ast::RdfLiteralAnnotation::Type(_, iri)) => {
				let iri = iri.build(scope)?;
				Ok(rdf.insert_literal(value, Cow::Owned(iri)))
			}
			Some(ast::RdfLiteralAnnotation::Language(_, lang_literal)) => {
				let lang = lang_literal.decode()?;
				let tag = LangTagBuf::new(lang)
					.map_err(|InvalidLangTag(t)| Error::Language(lang_literal.span(), t))?;
				Ok(rdf.insert_lang_string(value, tag))
			}
		}
	}
}

impl ast::Iri {
	fn build(&self, scope: &Scope) -> Result<IriBuf, Error> {
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
	fn build(&self, scope: &Scope) -> Result<IriBuf, Error> {
		let (prefix_name, suffix) = self.as_str().split_once(':').unwrap();
		let prefix = scope
			.get_prefix(prefix_name)
			.ok_or_else(|| Error::UnknownPrefix(self.span(), prefix_name.to_owned()))?;
		IriBuf::new(format!("{prefix}{suffix}")).map_err(|InvalidIri(i)| Error::Iri(self.span(), i))
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::InnerExpr {
	type Target = (InnerExpr<R, TypeVariableRef>, TypeVariableRef);

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		match self {
			Self::Var(ident) => {
				let x = scope
					.get(ident)?
					.as_variable()
					.ok_or_else(|| Error::NotAVariable(ident.clone()))?;
				let type_ = scope.type_of(x).unwrap();
				Ok((InnerExpr::Var(x), type_))
			}
			Self::Literal(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				let ty = context.type_variables.fresh(Type::Literal(ty));
				Ok((InnerExpr::Literal(e), ty))
			}
			Self::List(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				let ty = context.type_variables.fresh(Type::List(ty));
				Ok((InnerExpr::List(e), ty))
			}
			Self::Map(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				let ty = context.type_variables.fresh(Type::Struct(ty));
				Ok((InnerExpr::Map(e), ty))
			}
			Self::Match(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				let ty = context.type_variables.fresh(Type::Enum(ty));
				Ok((InnerExpr::Match(e), ty))
			}
			Self::Call(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				Ok((InnerExpr::Call(e), ty))
			}
		}
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::ListExpr {
	type Target = (expr::List<R, TypeVariableRef>, ty::List<TypeVariableRef>);

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		match &self.desc {
			ListDescription::Given(list) => {
				let mut items = Vec::with_capacity(list.len());
				let mut types = Vec::with_capacity(list.len());

				for item in list {
					let e = item.build(rdf, context, scope)?;
					types.push(e.type_);
					items.push(e);
				}

				Ok((
					expr::List::Explicit(expr::ExplicitList::new(items)),
					ty::List::from_prefix(types),
				))
			}
			ListDescription::All(_, bindings, mapping) => {
				let (bound, sub_scope) = bindings.build(rdf, context, scope)?;

				match bound.intro.as_slice() {
					&[item_ty] => {
						let body = Box::new(match mapping {
							Some(m) => m.expr.build(rdf, context, &sub_scope)?,
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
				let (bound, sub_scope) = bindings.build(rdf, context, scope)?;

				match bound.intro.as_slice() {
					&[item_ty] => {
						let body = Box::new(match mapping {
							Some(m) => m.expr.build(rdf, context, &sub_scope)?,
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
									first: rdf.insert_iri(Cow::Borrowed(RDF_FIRST)),
									rest: rdf.insert_iri(Cow::Borrowed(RDF_REST)),
									nil: rdf.insert_iri(Cow::Borrowed(RDF_NIL)),
								},
								head: Box::new(head.build(rdf, context, scope)?),
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
	type Target = (expr::Map<R, TypeVariableRef>, ty::Struct<TypeVariableRef>);

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		let mut fields = BTreeMap::new();
		let mut entries = TypedMap::new();

		for entry in &self.entries {
			let key = entry.key.build(rdf, context, scope)?;
			let value = entry.value.build(rdf, context, scope)?;

			fields.insert(
				key.type_,
				ty::Field {
					type_: value.type_,
					required: true,
				},
			);

			entries.insert(key, value);
		}

		let e = expr::Map { entries };

		let t = ty::Struct { fields };

		Ok((e, t))
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::ConstValue {
	type Target = TypedValue<R, TypeVariableRef>;

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		let (inner, type_) = match self {
			Self::Literal(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				let ty = context.type_variables.fresh(Type::Literal(ty));
				(TypedValueInnerDesc::Literal(e), ty)
			}
			Self::List(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				let ty = context.type_variables.fresh(Type::List(ty));
				(TypedValueInnerDesc::List(e), ty)
			}
			Self::Map(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				let ty = context.type_variables.fresh(Type::Struct(ty));
				(TypedValueInnerDesc::Map(e), ty)
			}
		};

		Ok(TypedValue {
			type_,
			desc: TypedValueDesc::Constant(inner),
		})
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::ConstList {
	type Target = (
		Vec<TypedValue<R, TypeVariableRef>>,
		ty::List<TypeVariableRef>,
	);

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		let mut items = Vec::with_capacity(self.items.len());
		let mut types = Vec::with_capacity(self.items.len());

		for item in &self.items {
			let e = item.build(rdf, context, scope)?;
			types.push(e.type_);
			items.push(e);
		}

		Ok((items, ty::List::from_prefix(types)))
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::ConstMap {
	type Target = (
		TypedMap<R, TypedValue<R, TypeVariableRef>, TypeVariableRef>,
		ty::Struct<TypeVariableRef>,
	);

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		let mut fields = BTreeMap::new();
		let mut entries = TypedMap::new();

		for entry in &self.entries {
			let key = entry.key.build(rdf, context, scope)?;
			let value = entry.value.build(rdf, context, scope)?;

			fields.insert(
				key.type_,
				ty::Field {
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
	type Target = (expr::Match<R, TypeVariableRef>, ty::Enum<TypeVariableRef>);

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		let mut t = ty::Enum {
			variants: BTreeMap::new(),
		};

		let mut e = expr::Match {
			cases: BTreeMap::new(),
			order: Vec::with_capacity(self.cases.len()),
		};

		for case in &self.cases {
			let value = case.value.build(rdf, context, scope)?;
			t.variants
				.insert(case.name.as_str().to_owned(), value.type_);
			e.cases.insert(case.name.as_str().to_owned(), value);
			e.order.push(case.name.as_str().to_owned());
		}

		Ok((e, t))
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::CallExpr {
	type Target = (expr::Call<R, TypeVariableRef>, TypeVariableRef);

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		let f_ref = scope
			.get(&self.function)?
			.as_function()
			.ok_or_else(|| Error::NotAFunction(self.function.clone()))?;

		let signature = context.get_function(f_ref).unwrap().signature.clone();

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
				let e = a.build(rdf, context, scope)?;
				context.type_variables.assert_subtype_of(e.type_, a_ty);
				Ok(e)
			})
			.collect::<Result<_, _>>()?;

		let e = expr::Call {
			function: f_ref,
			args,
		};

		let t = context.type_variables.fresh(TypeValue::Undefined);
		context
			.type_variables
			.assert_subtype_of(signature.return_type, t);

		Ok((e, t))
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::LiteralExpr {
	type Target = (Literal, ty::LiteralType);

	fn build<C>(
		&'ast self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut BuildContext<'_, '_, 'ast, R, C>,
		scope: &Scope,
	) -> Result<Self::Target, Error> {
		match self {
			Self::Unit(_) => Ok((Literal::Unit, ty::LiteralType::Unit)),
			Self::True(a) => {
				let (v, t) = a.build(rdf, context, scope)?;
				Ok((Literal::Boolean(v), ty::LiteralType::Boolean(t)))
			}
			Self::False(a) => {
				let (v, t) = a.build(rdf, context, scope)?;
				Ok((Literal::Boolean(v), ty::LiteralType::Boolean(t)))
			}
			Self::Number(a) => {
				let (v, t) = a.build(rdf, context, scope)?;
				Ok((Literal::Number(v), ty::LiteralType::Number(t)))
			}
			Self::TextString(a) => {
				let (v, t) = a.build(rdf, context, scope)?;
				Ok((Literal::TextString(v), ty::LiteralType::TextString(t)))
			}
		}
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::True {
	type Target = (bool, ty::Boolean);

	fn build<C>(
		&'ast self,
		_rdf: &mut impl RdfContextMut<R>,
		_context: &mut BuildContext<'_, '_, 'ast, R, C>,
		_scope: &Scope,
	) -> Result<Self::Target, Error> {
		Ok((true, ty::Boolean::singleton(true)))
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::False {
	type Target = (bool, ty::Boolean);

	fn build<C>(
		&'ast self,
		_rdf: &mut impl RdfContextMut<R>,
		_context: &mut BuildContext<'_, '_, 'ast, R, C>,
		_scope: &Scope,
	) -> Result<Self::Target, Error> {
		Ok((false, ty::Boolean::singleton(false)))
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::Number {
	type Target = (Number, ty::Number);

	fn build<C>(
		&'ast self,
		_rdf: &mut impl RdfContextMut<R>,
		_context: &mut BuildContext<'_, '_, 'ast, R, C>,
		_scope: &Scope,
	) -> Result<Self::Target, Error> {
		let value: Number = self
			.as_str()
			.parse()
			.map_err(|e| Error::Number(self.span(), e))?;
		Ok((value.clone(), ty::Number::singleton(value)))
	}
}

impl ast::TextString {
	pub fn decode(&self) -> Result<String, Error> {
		let s = self.as_str();
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
							.ok_or_else(|| Error::String(self.span()))?;

						let b = chars
							.next()
							.and_then(|c| c.to_digit(16))
							.ok_or_else(|| Error::String(self.span()))?;

						char::from_u32(a << 4 | b).ok_or_else(|| Error::String(self.span()))?
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
											| d.to_digit(16)
												.ok_or_else(|| Error::String(self.span()))?;
										i += 1
									}
									_ => return Err(Error::String(self.span())),
								}
							}

							char::from_u32(value).ok_or_else(|| Error::String(self.span()))?
						}
						_ => return Err(Error::String(self.span())),
					},
					Some('t') => '\t',
					Some('r') => '\r',
					Some('n') => '\n',
					Some('\'') => '\'',
					Some('"') => '"',
					Some('\\') => '\\',
					_ => return Err(Error::String(self.span())),
				},
				c => c,
			};

			value.push(decoded_c);
		}

		Ok(value)
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::TextString {
	type Target = (String, ty::TextString);

	fn build<C>(
		&'ast self,
		_rdf: &mut impl RdfContextMut<R>,
		_context: &mut BuildContext<'_, '_, 'ast, R, C>,
		_scope: &Scope,
	) -> Result<Self::Target, Error> {
		let value = self.decode()?;
		Ok((value.clone(), ty::TextString::singleton(value)))
	}
}
