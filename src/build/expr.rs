use std::{borrow::Cow, collections::BTreeMap, hash::Hash};

use btree_indexmap::BTreeIndexMap;
use iref::{InvalidIri, IriBuf};
use langtag::{InvalidLangTag, LangTagBuf};
use ll1::{Span, Spanned};
use rdf_types::{LiteralType, Quad, Term, RDF_FIRST, RDF_NIL, RDF_REST, XSD_STRING};

use crate::{
	expr::{self, Binding, Expr, InnerExpr, ListComponents},
	pattern::{CallPattern, MapPattern, PatternDesc},
	rdf::RdfContextMut,
	syntax::ast::{self, ListDescription},
	ty::{self, TypeDesc},
	value::{Map, Number, TypedValue, TypedValueDesc, TypedValueInnerDesc, TypedValueMap},
	Attributes, DatasetPattern, Literal, Pattern, Registry, TermPattern, Type, Value,
};

use super::{Build, BuildContext, BuildingPackage, Error, Scope, TypeValue, TypeVariableRef};

type BuiltBindings<'a, P, R> = (expr::Bindings<P, R, TypeVariableRef>, Scope<'a, P>);

type BuiltBindingsResult<'a, P, R> = Result<BuiltBindings<'a, P, R>, Error<P>>;

impl ast::Bindings {
	fn build<'ast, 'a, E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, E::PackageId>,
		scope: &'a Scope<E::PackageId>,
	) -> BuiltBindingsResult<'a, E::PackageId, E::Resource>
	where
		E: ?Sized + Registry,
		E::PackageId: Copy + Ord + Hash,
		E::Resource: Clone + Ord + Hash,
		D: RdfContextMut<Resource = E::Resource>,
	{
		let mut intro = Vec::with_capacity(self.list.len());
		let mut sub_scope = scope.begin();
		let mut variables = Vec::new();

		for binding in &self.list {
			let b = binding.build(context, namespace, scope, &mut variables, &mut sub_scope)?;
			intro.push(b);
		}

		let dataset = match &self.bound {
			Some(b) => b.dataset.build(context, namespace, &sub_scope)?,
			None => DatasetPattern::new(),
		};

		Ok((
			expr::Bindings {
				variables,
				list: intro,
				dataset,
			},
			sub_scope,
		))
	}
}

type BuiltBindingResult<P, R> = Result<Binding<P, R, TypeVariableRef>, Error<P>>;

impl ast::Binding {
	fn build<'ast, E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, E::PackageId>,
		scope: &Scope<E::PackageId>,
		variables: &mut Vec<TypeVariableRef>,
		sub_scope: &mut Scope<E::PackageId>,
	) -> BuiltBindingResult<E::PackageId, E::Resource>
	where
		E: ?Sized + Registry,
		E::PackageId: Copy + Ord + Hash,
		E::Resource: Clone + Ord + Hash,
		D: RdfContextMut<Resource = E::Resource>,
	{
		let assignment = self
			.assignment
			.as_ref()
			.map(|a| a.value.build(context, namespace, scope))
			.transpose()?;

		let r#type = match &assignment {
			Some(a) => a.r#type,
			None => context
				.type_variables
				.fresh(Type::resource(), Some(self.span())),
		};

		let pattern = self
			.pattern
			.build_pattern(context, namespace, scope, variables, sub_scope)?;

		context.type_variables.assert_eq(r#type, pattern.r#type);

		Ok(Binding {
			pattern,
			r#type,
			assignment,
		})
	}
}

type BuildPatternResult<P, R> = Result<Pattern<P, R, TypeVariableRef>, Error<P>>;

impl ast::Expr {
	fn build_pattern<'ast, E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, E::PackageId>,
		scope: &Scope<E::PackageId>,
		variables: &mut Vec<TypeVariableRef>,
		sub_scope: &mut Scope<E::PackageId>,
	) -> BuildPatternResult<E::PackageId, E::Resource>
	where
		E: ?Sized + Registry,
		E::PackageId: Copy + Ord + Hash,
		E::Resource: Clone + Ord + Hash,
		D: RdfContextMut<Resource = E::Resource>,
	{
		match self {
			Self::Var(ident) => {
				let ty = context
					.type_variables
					.fresh(TypeValue::Undefined, Some(ident.span()));
				variables.push(ty);
				let x = sub_scope.expr.insert(ident, ty);
				Ok(Pattern::new(ty, PatternDesc::Var(x)))
			}
			Self::List(list) => {
				let (ty, pattern) =
					list.build_pattern(context, namespace, scope, variables, sub_scope)?;
				Ok(Pattern::new(
					context.type_variables.fresh(
						Type::new(Attributes::new(), TypeDesc::List(ty)),
						Some(self.span()),
					),
					PatternDesc::List(pattern),
				))
			}
			Self::Map(map) => {
				let (ty, pattern) =
					map.build_pattern(context, namespace, scope, variables, sub_scope)?;
				Ok(Pattern::new(
					context.type_variables.fresh(
						Type::new(Attributes::new(), TypeDesc::Map(ty)),
						Some(self.span()),
					),
					PatternDesc::Map(pattern),
				))
			}
			Self::Call(call) => {
				let (ty, pattern) =
					call.build_pattern(context, namespace, scope, variables, sub_scope)?;
				Ok(Pattern::new(ty, PatternDesc::Call(pattern)))
			}
			_ => {
				todo!()
			}
		}
	}
}

type BuiltListPattern<P, R> = (
	ty::List<P, TypeVariableRef>,
	Vec<Pattern<P, R, TypeVariableRef>>,
);

type BuildListPatternResult<P, R> = Result<BuiltListPattern<P, R>, Error<P>>;

impl ast::ListExpr {
	fn build_pattern<'ast, E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, E::PackageId>,
		scope: &Scope<E::PackageId>,
		variables: &mut Vec<TypeVariableRef>,
		sub_scope: &mut Scope<E::PackageId>,
	) -> BuildListPatternResult<E::PackageId, E::Resource>
	where
		E: ?Sized + Registry,
		E::PackageId: Copy + Ord + Hash,
		E::Resource: Clone + Ord + Hash,
		D: RdfContextMut<Resource = E::Resource>,
	{
		match &self.desc {
			ast::ListDescription::Given(items) => {
				let mut patterns = Vec::with_capacity(items.len());
				let mut types = Vec::with_capacity(items.len());

				for item in items {
					let pattern =
						item.build_pattern(context, namespace, scope, variables, sub_scope)?;
					types.push(pattern.r#type);
					patterns.push(pattern);
				}

				Ok((ty::List::new(types, None), patterns))
			}
			_ => todo!(),
		}
	}
}

type BuiltMapPattern<P, R> = (
	ty::Map<P, R, TypeVariableRef>,
	MapPattern<P, R, TypeVariableRef>,
);

type BuildMapPatternResult<P, R> = Result<BuiltMapPattern<P, R>, Error<P>>;

impl ast::MapExpr {
	fn build_pattern<E, D>(
		&self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<E::PackageId>,
		scope: &Scope<E::PackageId>,
		variables: &mut Vec<TypeVariableRef>,
		sub_scope: &mut Scope<E::PackageId>,
	) -> BuildMapPatternResult<E::PackageId, E::Resource>
	where
		E: ?Sized + Registry,
		E::PackageId: Copy + Ord + Hash,
		E::Resource: Clone + Ord + Hash,
		D: RdfContextMut<Resource = E::Resource>,
	{
		let mut entries = MapPattern::new();
		let mut fields = BTreeIndexMap::new();

		for entry in &self.entries {
			let key = entry.key.build(context, namespace, scope)?;
			let value = entry
				.value
				.build_pattern(context, namespace, scope, variables, sub_scope)?;

			fields.insert(
				key.r#type,
				ty::map::Field::new(Attributes::new(), value.r#type, true),
			);

			entries.insert(key, value);
		}

		Ok((ty::Map::new(fields), entries))
	}
}

type BuiltCallPattern<P, R> = (TypeVariableRef, CallPattern<P, R, TypeVariableRef>);

type BuildCallPatternResult<P, R> = Result<BuiltCallPattern<P, R>, Error<P>>;

impl ast::CallExpr {
	fn build_pattern<E, D>(
		&self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<E::PackageId>,
		scope: &Scope<E::PackageId>,
		variables: &mut Vec<TypeVariableRef>,
		sub_scope: &mut Scope<E::PackageId>,
	) -> BuildCallPatternResult<E::PackageId, E::Resource>
	where
		E: ?Sized + Registry,
		E::PackageId: Copy + Ord + Hash,
		E::Resource: Clone + Ord + Hash,
		D: RdfContextMut<Resource = E::Resource>,
	{
		let path = self.function.build(context, namespace, scope)?;

		let f_ref = scope
			.get(context, namespace, &path)
			.ok_or_else(|| Error::UnknownFunction(self.function.span(), path.clone()))?
			.as_function()
			.ok_or_else(|| Error::NotAFunction(self.function.span(), path.clone()))?;

		let signature = namespace
			.import_function_signature(context, f_ref)
			.unwrap()
			.clone();

		if self.args.len() != signature.arity() {
			return Err(Error::ArgumentCount {
				span: self.span(),
				expected: signature.args.len(),
				found: self.args.len(),
			});
		}

		let mut ast_args = self.args.iter();

		let const_args = signature
			.const_args
			.iter()
			.map(|&a_ty| {
				let const_scope = scope.begin();
				let a = ast_args.next().unwrap();
				let e = a.build(context, namespace, &const_scope)?;
				context.type_variables.assert_subtype_of(e.r#type, a_ty);
				Ok(e)
			})
			.collect::<Result<_, _>>()?;

		let args = signature
			.args
			.iter()
			.map(|&a_ty| {
				let a = ast_args.next().unwrap();
				let e = a.build_pattern(context, namespace, scope, variables, sub_scope)?;
				context.type_variables.assert_eq(e.r#type, a_ty);
				Ok(e)
			})
			.collect::<Result<_, _>>()?;

		let e = CallPattern {
			function: f_ref,
			const_args,
			args,
		};

		let t = context
			.type_variables
			.fresh(TypeValue::Undefined, Some(self.span()));
		context.type_variables.assert_eq(signature.return_type, t);

		Ok((t, e))
	}
}

impl<'ast, P: Copy + PartialEq, R: Clone + Ord> Build<'ast, P, R> for ast::RdfDataset {
	type Target = DatasetPattern<R>;

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
		D: RdfContextMut<Resource = R>,
	{
		let mut result = DatasetPattern::new();

		for statement in &self.statements {
			statement.build(context, namespace, scope, &mut result)?;
		}

		Ok(result)
	}
}

impl ast::RdfStatement {
	fn build<E, D>(
		&self,
		context: &mut BuildContext<E, D>,
		namespace: &BuildingPackage<E::PackageId>,
		scope: &Scope<E::PackageId>,
		result: &mut DatasetPattern<E::Resource>,
	) -> Result<(), Error<E::PackageId>>
	where
		E: ?Sized + Registry,
		E::PackageId: Copy + PartialEq,
		E::Resource: Clone + Ord,
		D: RdfContextMut<Resource = E::Resource>,
	{
		let subject = self.subject.build(context, namespace, scope)?;

		for po in &self.predicates_objects {
			po.build(context, namespace, scope, result, &subject)?;
		}

		Ok(())
	}
}

impl ast::RdfPredicateObjects {
	fn build<E, D>(
		&self,
		context: &mut BuildContext<E, D>,
		namespace: &BuildingPackage<E::PackageId>,
		scope: &Scope<E::PackageId>,
		result: &mut DatasetPattern<E::Resource>,
		subject: &TermPattern<E::Resource>,
	) -> Result<(), Error<E::PackageId>>
	where
		E: ?Sized + Registry,
		E::PackageId: Copy + PartialEq,
		E::Resource: Clone + Ord,
		D: RdfContextMut<Resource = E::Resource>,
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
	fn build_unscoped<P>(&self) -> Result<Term, Error<P>> {
		match self {
			Self::Iri(i) => i.build_unscoped().map(Term::iri),
			Self::Literal(l) => l.build_unscoped().map(Term::Literal),
		}
	}

	fn build<E, D>(
		&self,
		context: &mut BuildContext<E, D>,
		namespace: &BuildingPackage<E::PackageId>,
		scope: &Scope<E::PackageId>,
	) -> Result<E::Resource, Error<E::PackageId>>
	where
		E: ?Sized + Registry,
		E::PackageId: Copy + PartialEq,
		E::Resource: Clone,
		D: RdfContextMut<Resource = E::Resource>,
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
	fn build<E, D>(
		&self,
		context: &mut BuildContext<E, D>,
		namespace: &BuildingPackage<E::PackageId>,
		scope: &Scope<E::PackageId>,
	) -> Result<TermPattern<E::Resource>, Error<E::PackageId>>
	where
		E: ?Sized + Registry,
		E::PackageId: Copy + PartialEq,
		E::Resource: Clone,
		D: RdfContextMut<Resource = E::Resource>,
	{
		match self {
			Self::Var(ident) => {
				let x = scope
					.get_by_name(context, namespace, ident.as_str())
					.ok_or_else(|| Error::<E::PackageId>::UnknownVariable(ident.clone()))?
					.as_variable()
					.ok_or_else(|| Error::<E::PackageId>::NotAVariable(ident.clone()))?;

				let t = context
					.type_variables
					.fresh(Type::resource(), Some(self.span()));
				context
					.type_variables
					.assert_eq(scope.expr.type_of(x).unwrap(), t);

				Ok(TermPattern::Var(x))
			}
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
	fn build_unscoped<P>(&self) -> Result<rdf_types::Literal, Error<P>> {
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

	fn build<E, D>(
		&self,
		context: &mut BuildContext<E, D>,
		namespace: &BuildingPackage<E::PackageId>,
		scope: &Scope<E::PackageId>,
	) -> Result<E::Resource, Error<E::PackageId>>
	where
		E: ?Sized + Registry,
		E::PackageId: Copy + PartialEq,
		E::Resource: Clone,
		D: RdfContextMut<Resource = E::Resource>,
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
	fn decode<P>(&self) -> Result<LangTagBuf, Error<P>> {
		LangTagBuf::new(self.as_str()[1..].to_owned())
			.map_err(|InvalidLangTag(t)| Error::Language(self.span(), t))
	}
}

impl ast::Iri {
	fn build_unscoped<P>(&self) -> Result<IriBuf, Error<P>> {
		match self {
			Self::Full(iri) => iri.decode(),
			Self::Compact(compact_iri) => {
				let prefix = compact_iri.as_str().split_once(':').unwrap().0;
				Err(Error::UnknownPrefix(compact_iri.span(), prefix.to_owned()))
			}
		}
	}

	fn build<E, D>(
		&self,
		context: &BuildContext<E, D>,
		namespace: &BuildingPackage<E::PackageId>,
		scope: &Scope<E::PackageId>,
	) -> Result<IriBuf, Error<E::PackageId>>
	where
		E: ?Sized + Registry,
		E::PackageId: Copy + PartialEq,
	{
		match self {
			Self::Full(iri) => iri.decode(),
			Self::Compact(compact_iri) => compact_iri.build(context, namespace, scope),
		}
	}
}

impl ast::CompactIri {
	fn build<E, D>(
		&self,
		context: &BuildContext<E, D>,
		namespace: &BuildingPackage<E::PackageId>,
		scope: &Scope<E::PackageId>,
	) -> Result<IriBuf, Error<E::PackageId>>
	where
		E: ?Sized + Registry,
		E::PackageId: Copy + PartialEq,
	{
		let (prefix_name, suffix) = self.as_str().split_once(':').unwrap();
		let prefix = scope
			.get_by_name(context, namespace, prefix_name)
			.ok_or_else(|| {
				Error::<E::PackageId>::UnknownPrefix(self.span(), prefix_name.to_owned())
			})?
			.as_prefix()
			.ok_or_else(|| {
				Error::<E::PackageId>::NotAPrefix(self.span(), prefix_name.to_owned())
			})?;
		IriBuf::new(format!("{prefix}{suffix}"))
			.map_err(|InvalidIri(i)| Error::<E::PackageId>::Iri(self.span(), i))
	}
}

impl<'ast, P: Copy + Ord + Hash, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::Expr {
	type Target = Expr<P, R, TypeVariableRef>;

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
		D: RdfContextMut<Resource = R>,
	{
		match self {
			Self::Let(_, bindings, _, inner) => {
				let (bindings, sub_scope) = bindings.build(context, namespace, scope)?;
				let inner = inner.build(context, namespace, &sub_scope)?;
				Ok(Expr {
					r#type: inner.r#type,
					inner: InnerExpr::Let(bindings, Box::new(inner)),
				})
			}
			Self::Var(ident) => {
				let x = scope
					.get_by_name(context, namespace, ident.as_str())
					.ok_or_else(|| Error::UnknownVariable(ident.clone()))?
					.as_variable()
					.ok_or_else(|| Error::NotAVariable(ident.clone()))?;
				let type_ = scope.expr.type_of(x).unwrap();
				Ok(Expr {
					r#type: type_,
					inner: InnerExpr::Var(x),
				})
			}
			Self::Resource(iri) => {
				let iri = iri.build(context, namespace, scope)?;
				let resource = context.rdf.insert_iri(Cow::Owned(iri));
				let type_ = context.type_variables.fresh(
					Type::new(
						Attributes::new(),
						TypeDesc::Resource(ty::Resource::singleton(resource.clone())),
					),
					Some(self.span()),
				);
				Ok(Expr {
					r#type: type_,
					inner: InnerExpr::Resource(resource),
				})
			}
			Self::Literal(expr) => {
				let (e, ty) = expr.build(context, namespace, scope)?;
				let type_ = context.type_variables.fresh(
					Type::new(Attributes::new(), TypeDesc::Literal(ty)),
					Some(self.span()),
				);
				Ok(Expr {
					r#type: type_,
					inner: InnerExpr::Literal(e),
				})
			}
			Self::List(expr) => {
				let (e, ty) = expr.build(context, namespace, scope)?;
				let type_ = context.type_variables.fresh(
					Type::new(Attributes::new(), TypeDesc::List(ty)),
					Some(self.span()),
				);
				Ok(Expr {
					r#type: type_,
					inner: InnerExpr::List(e),
				})
			}
			Self::Map(expr) => {
				let (e, ty) = expr.build(context, namespace, scope)?;
				let type_ = context.type_variables.fresh(
					Type::new(Attributes::new(), TypeDesc::Map(ty)),
					Some(self.span()),
				);
				Ok(Expr {
					r#type: type_,
					inner: InnerExpr::Map(e),
				})
			}
			Self::Match(expr) => {
				let (e, ty) = expr.build(context, namespace, scope)?;
				let type_ = context.type_variables.fresh(
					Type::new(Attributes::new(), TypeDesc::Enum(ty)),
					Some(self.span()),
				);
				Ok(Expr {
					r#type: type_,
					inner: InnerExpr::Match(e),
				})
			}
			Self::Call(expr) => {
				let (e, type_) = expr.build(context, namespace, scope)?;
				Ok(Expr {
					r#type: type_,
					inner: InnerExpr::Call(e),
				})
			}
			Self::Block(_, e, _) => e.build(context, namespace, scope),
		}
	}
}

impl<'ast, P: Copy + Ord + Hash, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::ListExpr {
	type Target = (
		expr::List<P, R, TypeVariableRef>,
		ty::List<P, TypeVariableRef>,
	);

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
		D: RdfContextMut<Resource = R>,
	{
		match &self.desc {
			ListDescription::Given(list) => {
				let mut items = Vec::with_capacity(list.len());
				let mut types = Vec::with_capacity(list.len());

				for item in list {
					let e = item.build(context, namespace, scope)?;
					types.push(e.r#type);
					items.push(e);
				}

				Ok((
					expr::List::Explicit(expr::ExplicitList::new(items)),
					ty::List::from_prefix(types),
				))
			}
			ListDescription::All(_, bindings, mapping) => {
				let (bound, sub_scope) = bindings.build(context, namespace, scope)?;

				match bound.list.as_slice() {
					[item] => {
						if item.assignment.is_some() {
							todo!()
						}

						let body = Box::new(match mapping {
							Some(m) => m.expr.build(context, namespace, &sub_scope)?,
							None => Expr {
								r#type: item.r#type,
								inner: InnerExpr::Var(sub_scope.expr.offset()),
							},
						});

						let resource_ty = context
							.type_variables
							.fresh(TypeValue::Pattern(Type::resource()), None);
						context.type_variables.assert_eq(item.r#type, resource_ty);

						let ty = ty::List::uniform(body.r#type);
						let e = expr::List::Implicit(expr::ImplicitList::Unordered(
							expr::UnorderedList { bound, body },
						));

						Ok((e, ty))
					}
					_ => Err(Error::ExpectedExactlyOneBinding(self.span())),
				}
			}
			ListDescription::List(_, bindings, _, head, mapping) => {
				let (bound, sub_scope) = bindings.build(context, namespace, scope)?;

				match bound.list.as_slice() {
					[item] => {
						if item.assignment.is_some() {
							todo!()
						}

						let body = Box::new(match mapping {
							Some(m) => m.expr.build(context, namespace, &sub_scope)?,
							None => Expr {
								r#type: item.r#type,
								inner: InnerExpr::Var(sub_scope.expr.offset()),
							},
						});

						let resource_ty = context
							.type_variables
							.fresh(TypeValue::Pattern(Type::resource()), None);
						context.type_variables.assert_eq(item.r#type, resource_ty);

						let head = head.build(context, namespace, scope)?;
						let resource_ty = context
							.type_variables
							.fresh(TypeValue::Pattern(Type::resource()), None);
						context.type_variables.assert_eq(head.r#type, resource_ty);

						let ty = ty::List::uniform(body.r#type);
						let e =
							expr::List::Implicit(expr::ImplicitList::Ordered(expr::OrderedList {
								components: ListComponents {
									first: context.rdf.insert_iri(Cow::Borrowed(RDF_FIRST)),
									rest: context.rdf.insert_iri(Cow::Borrowed(RDF_REST)),
									nil: context.rdf.insert_iri(Cow::Borrowed(RDF_NIL)),
								},
								head: Box::new(head),
								bound,
								body,
							}));

						Ok((e, ty))
					}
					_ => Err(Error::ExpectedExactlyOneBinding(self.span())),
				}
			}
		}
	}
}

impl<'ast, P: Copy + Ord + Hash, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::MapExpr {
	type Target = (
		expr::Map<P, R, TypeVariableRef>,
		ty::Map<P, R, TypeVariableRef>,
	);

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
		D: RdfContextMut<Resource = R>,
	{
		let mut fields = BTreeIndexMap::new();
		let mut entries = TypedValueMap::new();

		for entry in &self.entries {
			let key = entry.key.build(context, namespace, scope)?;
			let value = entry.value.build(context, namespace, scope)?;

			fields.insert(
				key.r#type,
				ty::map::Field::new(
					entry.metadata.build(context, namespace, scope)?,
					value.r#type,
					entry.optional.is_none(),
				),
			);

			entries.insert(key, value);
		}

		let e = expr::Map { entries };

		let t = ty::Map { fields };

		Ok((e, t))
	}
}

impl ast::ConstValue {
	pub fn build_unscoped<P>(&self) -> Result<Value, Error<P>> {
		match self {
			Self::Resource(term) => term.build_unscoped().map(Value::Resource),
			Self::Literal(expr) => expr.build_unscoped().map(Value::Literal),
			Self::List(expr) => expr.build_unscoped().map(Value::List),
			Self::Map(expr) => expr.build_unscoped().map(Value::Map),
		}
	}
}

impl<'ast, P: Copy + PartialEq, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::ConstValue {
	type Target = TypedValue<P, R, TypeVariableRef>;

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
		D: RdfContextMut<Resource = R>,
	{
		let (inner, type_) = match self {
			Self::Resource(term) => {
				let term = term.build(context, namespace, scope)?;
				let ty = context.type_variables.fresh(
					Type::new(
						Attributes::new(),
						TypeDesc::Resource(ty::Resource::singleton(term.clone())),
					),
					Some(self.span()),
				);
				(TypedValueInnerDesc::Resource(term), ty)
			}
			Self::Literal(expr) => {
				let (e, ty) = expr.build(context, namespace, scope)?;
				let ty = context.type_variables.fresh(
					Type::new(Attributes::new(), TypeDesc::Literal(ty)),
					Some(self.span()),
				);
				(TypedValueInnerDesc::Literal(e), ty)
			}
			Self::List(expr) => {
				let (e, ty) = expr.build(context, namespace, scope)?;
				let ty = context.type_variables.fresh(
					Type::new(Attributes::new(), TypeDesc::List(ty)),
					Some(self.span()),
				);
				(TypedValueInnerDesc::List(e), ty)
			}
			Self::Map(expr) => {
				let (e, ty) = expr.build(context, namespace, scope)?;
				let ty = context.type_variables.fresh(
					Type::new(Attributes::new(), TypeDesc::Map(ty)),
					Some(self.span()),
				);
				(TypedValueInnerDesc::Map(e), ty)
			}
		};

		Ok(TypedValue {
			r#type: type_,
			desc: TypedValueDesc::Constant(inner),
		})
	}
}

impl ast::ConstList {
	pub fn build_unscoped<P>(&self) -> Result<Vec<Value>, Error<P>> {
		self.items
			.iter()
			.map(|v| v.build_unscoped())
			.collect::<Result<_, _>>()
	}
}

impl<'ast, P: Copy + PartialEq, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::ConstList {
	type Target = (
		Vec<TypedValue<P, R, TypeVariableRef>>,
		ty::List<P, TypeVariableRef>,
	);

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
		D: RdfContextMut<Resource = R>,
	{
		let mut items = Vec::with_capacity(self.items.len());
		let mut types = Vec::with_capacity(self.items.len());

		for item in &self.items {
			let e = item.build(context, namespace, scope)?;
			types.push(e.r#type);
			items.push(e);
		}

		Ok((items, ty::List::from_prefix(types)))
	}
}

impl ast::ConstMap {
	pub fn build_unscoped<P>(&self) -> Result<Map, Error<P>> {
		self.entries
			.iter()
			.map(|entry| Ok((entry.key.build_unscoped()?, entry.value.build_unscoped()?)))
			.collect::<Result<_, _>>()
	}
}

impl<'ast, P: Copy + PartialEq, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::ConstMap {
	type Target = (
		TypedValueMap<P, R, TypedValue<P, R, TypeVariableRef>, TypeVariableRef>,
		ty::Map<P, R, TypeVariableRef>,
	);

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
		D: RdfContextMut<Resource = R>,
	{
		let mut fields = BTreeIndexMap::new();
		let mut entries = TypedValueMap::new();

		for entry in &self.entries {
			let key = entry.key.build(context, namespace, scope)?;
			let value = entry.value.build(context, namespace, scope)?;

			fields.insert(
				key.r#type,
				ty::map::Field::new(Attributes::new(), value.r#type, true),
			);

			entries.insert(key, value);
		}

		let t = ty::Map { fields };

		Ok((entries, t))
	}
}

impl<'ast, P: Copy + Ord + Hash, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::MatchExpr {
	type Target = (
		expr::Match<P, R, TypeVariableRef>,
		ty::Enum<P, R, TypeVariableRef>,
	);

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
		D: RdfContextMut<Resource = R>,
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
			let variant = ty::r#enum::Variant::new(
				case.metadata.build(context, namespace, scope)?,
				value.r#type,
			);
			t.variants.insert(case.name.as_str().to_owned(), variant);
			e.cases.insert(case.name.as_str().to_owned(), value);
			e.order.push(case.name.as_str().to_owned());
		}

		Ok((e, t))
	}
}

impl<'ast, P: Copy + Ord + Hash, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::CallExpr {
	type Target = (expr::Call<P, R, TypeVariableRef>, TypeVariableRef);

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
		D: RdfContextMut<Resource = R>,
	{
		let path = self.function.build(context, namespace, scope)?;

		let f_ref = scope
			.get(context, namespace, &path)
			.ok_or_else(|| Error::UnknownFunction(self.function.span(), path.clone()))?
			.as_function()
			.ok_or_else(|| Error::NotAFunction(self.function.span(), path.clone()))?;

		let signature = namespace
			.import_function_signature(context, f_ref)
			.unwrap()
			.clone();

		if self.args.len() != signature.arity() {
			return Err(Error::ArgumentCount {
				span: self.span(),
				expected: signature.args.len(),
				found: self.args.len(),
			});
		}

		let mut ast_args = self.args.iter();

		let const_args = signature
			.const_args
			.iter()
			.map(|&a_ty| {
				let const_scope = scope.begin();
				let a = ast_args.next().unwrap();
				let e = a.build(context, namespace, &const_scope)?;
				context.type_variables.assert_subtype_of(e.r#type, a_ty);
				Ok(e)
			})
			.collect::<Result<_, _>>()?;

		let args = signature
			.args
			.iter()
			.map(|&a_ty| {
				let a = ast_args.next().unwrap();
				let e = a.build(context, namespace, scope)?;
				context.type_variables.assert_subtype_of(e.r#type, a_ty);
				Ok(e)
			})
			.collect::<Result<_, _>>()?;

		let e = expr::Call {
			function: f_ref,
			const_args,
			args,
		};

		let t = context
			.type_variables
			.fresh(TypeValue::Undefined, Some(self.span()));

		context.type_variables.assert_eq(signature.return_type, t);

		Ok((e, t))
	}
}

impl ast::LiteralExpr {
	pub fn build_unscoped<P>(&self) -> Result<Literal, Error<P>> {
		match self {
			Self::Unit(_) => Ok(Literal::Unit),
			Self::True(_) => Ok(Literal::Boolean(true)),
			Self::False(_) => Ok(Literal::Boolean(false)),
			Self::Number(a) => a.build_unscoped().map(Literal::Number),
			Self::TextString(a) => a.decode().map(Literal::TextString),
		}
	}
}

impl<'ast, P, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::LiteralExpr {
	type Target = (Literal, ty::LiteralType);

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
		D: RdfContextMut<Resource = R>,
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

impl<'ast, P, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::True {
	type Target = (bool, ty::Boolean);

	fn build<E, D>(
		&'ast self,
		_context: &mut BuildContext<'_, E, D>,
		_namespace: &BuildingPackage<'ast, P>,
		_scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
	{
		Ok((true, ty::Boolean::singleton(true)))
	}
}

impl<'ast, P, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::False {
	type Target = (bool, ty::Boolean);

	fn build<E, D>(
		&'ast self,
		_context: &mut BuildContext<'_, E, D>,
		_namespace: &BuildingPackage<'ast, P>,
		_scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
	{
		Ok((false, ty::Boolean::singleton(false)))
	}
}

impl ast::Number {
	pub fn build_unscoped<P>(&self) -> Result<Number, Error<P>> {
		self.as_str()
			.parse()
			.map_err(|e| Error::Number(self.span(), e))
	}
}

impl<'ast, P, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::Number {
	type Target = (Number, ty::Number);

	fn build<E, D>(
		&'ast self,
		_context: &mut BuildContext<'_, E, D>,
		_namespace: &BuildingPackage<'ast, P>,
		_scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
	{
		let value = self.build_unscoped()?;
		Ok((value.clone(), ty::Number::singleton(value)))
	}
}

impl ast::TextString {
	pub fn decode<P>(&self) -> Result<String, Error<P>> {
		decode_string_literal(self.as_str(), self.span())
	}
}

impl<'ast, P, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::TextString {
	type Target = (String, ty::TextString);

	fn build<E, D>(
		&'ast self,
		_context: &mut BuildContext<'_, E, D>,
		_namespace: &BuildingPackage<'ast, P>,
		_scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
	{
		let value = self.decode()?;
		Ok((value.clone(), ty::TextString::singleton(value)))
	}
}

impl ast::RdfString {
	pub fn decode<P>(&self) -> Result<String, Error<P>> {
		decode_string_literal(self.as_str(), self.span())
	}
}

impl<'ast, P, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::RdfString {
	type Target = (String, ty::TextString);

	fn build<E, D>(
		&'ast self,
		_context: &mut BuildContext<'_, E, D>,
		_namespace: &BuildingPackage<'ast, P>,
		_scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P, Resource = R>,
	{
		let value = self.decode()?;
		Ok((value.clone(), ty::TextString::singleton(value)))
	}
}

fn decode_string_literal<P>(s: &str, span: Span) -> Result<String, Error<P>> {
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
						.ok_or(Error::String(span))?;

					let b = chars
						.next()
						.and_then(|c| c.to_digit(16))
						.ok_or(Error::String(span))?;

					char::from_u32(a << 4 | b).ok_or(Error::String(span))?
				}
				Some('u' | 'U') => match chars.next() {
					Some('{') => {
						let mut value = 0;
						let mut i = 0;

						loop {
							match chars.next() {
								Some('}') if i >= 2 => break,
								Some(d) if i < 6 => {
									value =
										value << 4 | d.to_digit(16).ok_or(Error::String(span))?;
									i += 1
								}
								_ => return Err(Error::String(span)),
							}
						}

						char::from_u32(value).ok_or(Error::String(span))?
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
