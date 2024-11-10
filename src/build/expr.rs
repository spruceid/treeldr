use std::{borrow::Cow, collections::BTreeMap, hash::Hash};

use iref::{InvalidIri, IriBuf};
use langtag::{InvalidLangTag, LangTagBuf};
use ll1::{Span, Spanned};
use rdf_types::{LiteralType, Quad, Term, RDF_FIRST, RDF_NIL, RDF_REST, XSD_STRING};

use crate::{
	expr::{self, Bound, Expr, InnerExpr, ListComponents},
	rdf::RdfContextMut,
	syntax::ast::{self, ListDescription},
	ty::{self, TypeDesc},
	value::{Number, TypedMap, TypedValue, TypedValueDesc, TypedValueInnerDesc},
	Attributes, DatasetPattern, Literal, TermPattern, Type, Value,
};

use super::{Build, BuildContext, BuildingNamespace, Error, Scope, TypeValue, TypeVariableRef};

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::Expr {
	type Target = Expr<R, TypeVariableRef>;

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let (bound, inner, type_) = match self {
			Self::Let(_, bindings, _, inner) => {
				let (bound, sub_scope) = bindings.build(context, namespace, scope)?;
				let (inner, type_) = inner.build(context, namespace, &sub_scope)?;
				(bound, inner, type_)
			}
			Self::Inner(inner) => {
				let (inner, type_) = inner.build(context, namespace, scope)?;
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
		namespace: &BuildingNamespace<'ast>,
		scope: &'a Scope,
	) -> Result<(expr::Bound<R, TypeVariableRef>, Scope<'a>), Error>
	where
		R: Clone + Ord + Hash,
		D: RdfContextMut<R>,
	{
		let mut intro = Vec::with_capacity(self.variables.len());
		let mut sub_scope = scope.begin();

		for ident in &self.variables {
			let ty_ref = context.type_variables.fresh(Type::resource());
			intro.push(ty_ref);
			sub_scope.expr.insert(ident, ty_ref);
		}

		let dataset = match &self.bound {
			Some(b) => b.dataset.build(context, namespace, &sub_scope)?,
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
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let mut result = DatasetPattern::new();

		for statement in &self.statements {
			statement.build(context, namespace, scope, &mut result)?;
		}

		Ok(result)
	}
}

impl ast::RdfStatement {
	fn build<R, D>(
		&self,
		context: &mut BuildContext<R, D>,
		namespace: &BuildingNamespace,
		scope: &Scope,
		result: &mut DatasetPattern<R>,
	) -> Result<(), Error>
	where
		R: Clone + Ord,
		D: RdfContextMut<R>,
	{
		let subject = self.subject.build(context, namespace, scope)?;

		for po in &self.predicates_objects {
			po.build(context, namespace, scope, result, &subject)?;
		}

		Ok(())
	}
}

impl ast::RdfPredicateObjects {
	fn build<R, D>(
		&self,
		context: &mut BuildContext<R, D>,
		namespace: &BuildingNamespace,
		scope: &Scope,
		result: &mut DatasetPattern<R>,
		subject: &TermPattern<R>,
	) -> Result<(), Error>
	where
		R: Clone + Ord,
		D: RdfContextMut<R>,
	{
		let predicate = self.predicate.build(context, namespace, scope)?;

		for o in &self.objects {
			let object = o.build(context, namespace, scope)?;
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

	fn build<R, D>(
		&self,
		context: &mut BuildContext<R, D>,
		namespace: &BuildingNamespace,
		scope: &Scope,
	) -> Result<R, Error>
	where
		R: Clone,
		D: RdfContextMut<R>,
	{
		match self {
			Self::Iri(i) => {
				let iri = i.build(context, namespace, scope)?;
				Ok(context.rdf.insert_iri(Cow::Owned(iri)))
			}
			Self::Literal(l) => l.build(context, namespace, scope),
		}
	}
}

impl ast::RdfPattern {
	fn build<R, D>(
		&self,
		context: &mut BuildContext<R, D>,
		namespace: &BuildingNamespace,
		scope: &Scope,
	) -> Result<TermPattern<R>, Error>
	where
		R: Clone,
		D: RdfContextMut<R>,
	{
		match self {
			Self::Var(ident) => scope
				.get_by_name(context, namespace, ident.as_str())
				.ok_or_else(|| Error::UnknownVariable(ident.clone()))?
				.as_variable()
				.ok_or_else(|| Error::NotAVariable(ident.clone()))
				.map(TermPattern::Var),
			Self::Iri(i) => {
				let iri = i.build(context, namespace, scope)?;
				Ok(TermPattern::Resource(
					context.rdf.insert_iri(Cow::Owned(iri)),
				))
			}
			Self::Literal(l) => l
				.build(context, namespace, scope)
				.map(TermPattern::Resource),
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

	fn build<R, D>(
		&self,
		context: &mut BuildContext<R, D>,
		namespace: &BuildingNamespace,
		scope: &Scope,
	) -> Result<R, Error>
	where
		R: Clone,
		D: RdfContextMut<R>,
	{
		let value = self.value.decode()?;
		match &self.annotation {
			None => Ok(context.rdf.insert_literal(value, Cow::Borrowed(XSD_STRING))),
			Some(ast::RdfLiteralAnnotation::Type(_, iri)) => {
				let iri = iri.build(context, namespace, scope)?;
				Ok(context.rdf.insert_literal(value, Cow::Owned(iri)))
			}
			Some(ast::RdfLiteralAnnotation::Language(lang)) => {
				let tag = lang.decode()?;
				Ok(context.rdf.insert_lang_string(value, tag))
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

	fn build<R, D>(
		&self,
		context: &BuildContext<R, D>,
		namespace: &BuildingNamespace,
		scope: &Scope,
	) -> Result<IriBuf, Error> {
		match self {
			Self::Full(iri) => iri.decode(),
			Self::Compact(compact_iri) => compact_iri.build(context, namespace, scope),
		}
	}
}

impl ast::CompactIri {
	fn build<R, D>(
		&self,
		context: &BuildContext<R, D>,
		namespace: &BuildingNamespace,
		scope: &Scope,
	) -> Result<IriBuf, Error> {
		let (prefix_name, suffix) = self.as_str().split_once(':').unwrap();
		let prefix = scope
			.get_by_name(context, namespace, prefix_name)
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
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		match self {
			Self::Var(ident) => {
				let x = scope
					.get_by_name(context, namespace, ident.as_str())
					.ok_or_else(|| Error::UnknownVariable(ident.clone()))?
					.as_variable()
					.ok_or_else(|| Error::NotAVariable(ident.clone()))?;
				let type_ = scope.expr.type_of(x).unwrap();
				Ok((InnerExpr::Var(x), type_))
			}
			Self::Literal(expr) => {
				let (e, ty) = expr.build(context, namespace, scope)?;
				let ty = context
					.type_variables
					.fresh(Type::new(Attributes::new(), TypeDesc::Literal(ty)));
				Ok((InnerExpr::Literal(e), ty))
			}
			Self::List(expr) => {
				let (e, ty) = expr.build(context, namespace, scope)?;
				let ty = context
					.type_variables
					.fresh(Type::new(Attributes::new(), TypeDesc::List(ty)));
				Ok((InnerExpr::List(e), ty))
			}
			Self::Map(expr) => {
				let (e, ty) = expr.build(context, namespace, scope)?;
				let ty = context
					.type_variables
					.fresh(Type::new(Attributes::new(), TypeDesc::Struct(ty)));
				Ok((InnerExpr::Map(e), ty))
			}
			Self::Match(expr) => {
				let (e, ty) = expr.build(context, namespace, scope)?;
				let ty = context
					.type_variables
					.fresh(Type::new(Attributes::new(), TypeDesc::Enum(ty)));
				Ok((InnerExpr::Match(e), ty))
			}
			Self::Call(expr) => {
				let (e, ty) = expr.build(context, namespace, scope)?;
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
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		match &self.desc {
			ListDescription::Given(list) => {
				let mut items = Vec::with_capacity(list.len());
				let mut types = Vec::with_capacity(list.len());

				for item in list {
					let e = item.build(context, namespace, scope)?;
					types.push(e.type_);
					items.push(e);
				}

				Ok((
					expr::List::Explicit(expr::ExplicitList::new(items)),
					ty::List::from_prefix(types),
				))
			}
			ListDescription::All(_, bindings, mapping) => {
				let (bound, sub_scope) = bindings.build(context, namespace, scope)?;

				match bound.intro.as_slice() {
					&[item_ty] => {
						let body = Box::new(match mapping {
							Some(m) => m.expr.build(context, namespace, &sub_scope)?,
							None => Expr {
								type_: item_ty,
								bound: Bound::default(),
								inner: InnerExpr::Var(sub_scope.expr.offset()),
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
				let (bound, sub_scope) = bindings.build(context, namespace, scope)?;

				match bound.intro.as_slice() {
					&[item_ty] => {
						let body = Box::new(match mapping {
							Some(m) => m.expr.build(context, namespace, &sub_scope)?,
							None => Expr {
								type_: item_ty,
								bound: Bound::default(),
								inner: InnerExpr::Var(sub_scope.expr.offset()),
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
								head: Box::new(head.build(context, namespace, scope)?),
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
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let mut fields = BTreeMap::new();
		let mut entries = TypedMap::new();

		for entry in &self.entries {
			let key = entry.key.build(context, namespace, scope)?;
			let value = entry.value.build(context, namespace, scope)?;

			fields.insert(
				key.type_,
				ty::Field {
					attributes: entry.metadata.build(context, namespace, scope)?,
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
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let (inner, type_) = match self {
			Self::Resource(term) => {
				let term = term.build(context, namespace, scope)?;
				let ty = context.type_variables.fresh(Type::new(
					Attributes::new(),
					TypeDesc::Resource(ty::Resource::singleton(term.clone())),
				));
				(TypedValueInnerDesc::Resource(term), ty)
			}
			Self::Literal(expr) => {
				let (e, ty) = expr.build(context, namespace, scope)?;
				let ty = context
					.type_variables
					.fresh(Type::new(Attributes::new(), TypeDesc::Literal(ty)));
				(TypedValueInnerDesc::Literal(e), ty)
			}
			Self::List(expr) => {
				let (e, ty) = expr.build(context, namespace, scope)?;
				let ty = context
					.type_variables
					.fresh(Type::new(Attributes::new(), TypeDesc::List(ty)));
				(TypedValueInnerDesc::List(e), ty)
			}
			Self::Map(expr) => {
				let (e, ty) = expr.build(context, namespace, scope)?;
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
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let mut items = Vec::with_capacity(self.items.len());
		let mut types = Vec::with_capacity(self.items.len());

		for item in &self.items {
			let e = item.build(context, namespace, scope)?;
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
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let mut fields = BTreeMap::new();
		let mut entries = TypedMap::new();

		for entry in &self.entries {
			let key = entry.key.build(context, namespace, scope)?;
			let value = entry.value.build(context, namespace, scope)?;

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
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
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
			let value = case.value.build(context, namespace, scope)?;
			let variant = ty::Variant {
				attributes: case.metadata.build(context, namespace, scope)?,
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
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		let path = self.function.build(context, namespace, scope)?;

		let f_ref = scope
			.get(context, namespace, &path)
			.ok_or_else(|| Error::UnknownFunction(path.clone()))?
			.as_function()
			.ok_or_else(|| Error::NotAFunction(path.clone()))?;

		let signature = namespace
			.import_function_signature(context, f_ref)
			.unwrap()
			.clone();

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
				let e = a.build(context, namespace, scope)?;
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
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		match self {
			Self::Unit(_) => Ok((Literal::Unit, ty::LiteralType::Unit)),
			Self::True(a) => {
				let (v, t) = a.build(context, namespace, scope)?;
				Ok((Literal::Boolean(v), ty::LiteralType::Boolean(t)))
			}
			Self::False(a) => {
				let (v, t) = a.build(context, namespace, scope)?;
				Ok((Literal::Boolean(v), ty::LiteralType::Boolean(t)))
			}
			Self::Number(a) => {
				let (v, t) = a.build(context, namespace, scope)?;
				Ok((Literal::Number(v), ty::LiteralType::Number(t)))
			}
			Self::TextString(a) => {
				let (v, t) = a.build(context, namespace, scope)?;
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
		_namespace: &BuildingNamespace<'ast>,
		_scope: &Scope,
	) -> Result<Self::Target, Error> {
		Ok((true, ty::Boolean::singleton(true)))
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::False {
	type Target = (bool, ty::Boolean);

	fn build<D>(
		&'ast self,
		_context: &mut BuildContext<'_, R, D>,
		_namespace: &BuildingNamespace<'ast>,
		_scope: &Scope,
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
		_namespace: &BuildingNamespace<'ast>,
		_scope: &Scope,
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
		_namespace: &BuildingNamespace<'ast>,
		_scope: &Scope,
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
		_namespace: &BuildingNamespace<'ast>,
		_scope: &Scope,
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
