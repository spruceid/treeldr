use std::hash::Hash;

use ll1::Spanned;

use crate::{
	rdf::RdfContextMut,
	syntax::ast,
	ty::{self, TypeDesc},
	Attributes, ModuleRef, Type,
};

use super::{
	Build, BuildContext, BuildEnvironment, BuildingPackage, Error, ExprScope, Scope,
	TypeVariableRef,
};

pub struct TypeDefinition<'ast, P> {
	pub module: ModuleRef<P>,
	pub metadata: &'ast ast::Metadata,
	pub value: &'ast ast::TypeDefinition,
	pub variable: TypeVariableRef,
}

impl<'ast, P> TypeDefinition<'ast, P> {
	pub fn build<E, D>(
		&self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
	) -> Result<TypeVariableRef, Error<P>>
	where
		E: BuildEnvironment<PackageId = P>,
		E::PackageId: Copy + Ord + Hash,
		E::Resource: Clone + Ord + Hash,
		D: RdfContextMut<Resource = E::Resource>,
	{
		let scope = Scope {
			module: Some(self.module),
			expr: ExprScope::new(None),
			const_: false,
		};

		let attributes = self.metadata.build(context, namespace, &scope)?;

		let y = self
			.value
			.value
			.build_with_attributes(context, namespace, &scope, attributes)?;

		context.type_variables.assert_eq(self.variable, y);
		Ok(self.variable)
	}
}

impl<'ast, P: Copy + Ord + Hash, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::TypeExpr {
	type Target = TypeVariableRef;

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: BuildEnvironment<PackageId = P, Resource = R>,
		D: RdfContextMut<Resource = R>,
	{
		self.build_with_attributes(context, namespace, scope, Attributes::new())
	}
}

impl ast::TypeExpr {
	pub fn build_with_attributes<'ast, E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, E::PackageId>,
		scope: &Scope<E::PackageId>,
		attributes: Attributes<E::Resource>,
	) -> Result<TypeVariableRef, Error<E::PackageId>>
	where
		E: BuildEnvironment,
		E::PackageId: Copy + Ord + Hash,
		E::Resource: Clone + Ord + Hash,
		D: RdfContextMut<Resource = E::Resource>,
	{
		match self {
			Self::Path(path_ast) => {
				let path = path_ast.build(context, namespace, scope)?;

				scope
					.get(context, namespace, &path)
					.ok_or_else(|| Error::UnknownType(path_ast.span(), path.clone()))?
					.into_owned()
					.import_type(context.environment.registry(), &mut context.type_variables)
					.ok_or_else(|| Error::NotAType(path_ast.span(), path.clone()))
			}
			Self::Literal(expr) => {
				let (_, ty) = expr.build(context, namespace, scope)?;
				Ok(context.type_variables.fresh(
					Type::new(attributes, TypeDesc::Literal(ty)),
					Some(expr.span()),
				))
			}
			Self::List(expr) => {
				let ty = expr.build(context, namespace, scope)?;
				Ok(context
					.type_variables
					.fresh(Type::new(attributes, TypeDesc::List(ty)), Some(expr.span())))
			}
			Self::Map(expr) => {
				let ty = expr.build(context, namespace, scope)?;
				Ok(context
					.type_variables
					.fresh(Type::new(attributes, TypeDesc::Map(ty)), Some(expr.span())))
			}
			Self::Enum(expr) => {
				let ty = expr.build(context, namespace, scope)?;
				Ok(context
					.type_variables
					.fresh(Type::new(attributes, TypeDesc::Enum(ty)), Some(expr.span())))
			}
		}
	}
}

impl<'ast, P: Copy + Ord + Hash, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::ListTypeExpr {
	type Target = ty::List<P, TypeVariableRef>;

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: BuildEnvironment<PackageId = P, Resource = R>,
		D: RdfContextMut<Resource = R>,
	{
		Ok(ty::List::new(
			self.prefix
				.iter()
				.map(|e| e.build(context, namespace, scope))
				.collect::<Result<_, _>>()?,
			self.rest
				.as_ref()
				.map(|r| r.r#type.build(context, namespace, scope))
				.transpose()?,
		))
	}
}

impl<'ast, P: Copy + Ord + Hash, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::MapTypeExpr {
	type Target = ty::Map<P, R, TypeVariableRef>;

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: BuildEnvironment<PackageId = P, Resource = R>,
		D: RdfContextMut<Resource = R>,
	{
		Ok(ty::Map {
			fields: self
				.entries
				.iter()
				.map(|f| {
					Ok((
						f.key.build(context, namespace, scope)?,
						ty::map::Field::new(
							f.metadata.build(context, namespace, scope)?,
							f.value.build(context, namespace, scope)?,
							self.optional.is_none(),
						),
					))
				})
				.collect::<Result<_, _>>()?,
		})
	}
}

impl<'ast, P: Copy + Ord + Hash, R: Clone + Ord + Hash> Build<'ast, P, R> for ast::EnumTypeExpr {
	type Target = ty::Enum<P, R, TypeVariableRef>;

	fn build<E, D>(
		&'ast self,
		context: &mut BuildContext<'_, E, D>,
		namespace: &BuildingPackage<'ast, P>,
		scope: &Scope<P>,
	) -> Result<Self::Target, Error<P>>
	where
		E: BuildEnvironment<PackageId = P, Resource = R>,
		D: RdfContextMut<Resource = R>,
	{
		Ok(ty::Enum {
			variants: self
				.variants
				.iter()
				.map(|v| {
					let variant = ty::r#enum::Variant::new(
						v.metadata.build(context, namespace, scope)?,
						v.r#type.build(context, namespace, scope)?,
					);

					Ok((v.ident.as_str().to_owned(), variant))
				})
				.collect::<Result<_, _>>()?,
		})
	}
}
