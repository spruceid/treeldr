use std::hash::Hash;

use crate::{
	rdf::RdfContextMut,
	syntax::ast,
	ty::{self, TypeDesc},
	Attributes, ModuleRef, Type,
};

use super::{Build, BuildContext, BuildingNamespace, Error, ExprScope, Scope, TypeVariableRef};

pub struct TypeDefinition<'ast> {
	pub module: ModuleRef,
	pub metadata: &'ast ast::Metadata,
	pub value: &'ast ast::TypeDefinition,
	pub variable: TypeVariableRef,
}

impl<'ast> TypeDefinition<'ast> {
	pub fn build<R, D>(
		&self,
		context: &mut BuildContext<'_, R, D>,
		namespace: &BuildingNamespace<'ast>,
	) -> Result<TypeVariableRef, Error>
	where
		R: Clone + Ord + Hash,
		D: RdfContextMut<R>,
	{
		let scope = Scope {
			module: Some(self.module),
			expr: ExprScope::new(None),
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

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::TypeExpr {
	type Target = TypeVariableRef;

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		self.build_with_attributes(context, namespace, scope, Attributes::new())
	}
}

impl ast::TypeExpr {
	pub fn build_with_attributes<'ast, R, D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
		attributes: Attributes<R>,
	) -> Result<TypeVariableRef, Error>
	where
		R: Clone + Ord + Hash,
		D: RdfContextMut<R>,
	{
		match self {
			Self::Path(path) => {
				let path = path.build(context, namespace, scope)?;

				scope
					.get(context, namespace, &path)
					.ok_or_else(|| Error::UnknownType(path.clone()))?
					.into_owned()
					.import_type(context.registry, &mut context.type_variables)
					.ok_or_else(|| Error::NotAType(path.clone()))
			}
			Self::Literal(expr) => {
				let (_, ty) = expr.build(context, namespace, scope)?;
				Ok(context
					.type_variables
					.fresh(Type::new(attributes, TypeDesc::Literal(ty))))
			}
			Self::List(expr) => {
				let ty = expr.build(context, namespace, scope)?;
				Ok(context
					.type_variables
					.fresh(Type::new(attributes, TypeDesc::List(ty))))
			}
			Self::Map(expr) => {
				let ty = expr.build(context, namespace, scope)?;
				Ok(context
					.type_variables
					.fresh(Type::new(attributes, TypeDesc::Struct(ty))))
			}
			Self::Enum(expr) => {
				let ty = expr.build(context, namespace, scope)?;
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
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		Ok(ty::List {
			prefix: self
				.prefix
				.iter()
				.map(|e| e.build(context, namespace, scope))
				.collect::<Result<_, _>>()?,
			rest: self
				.rest
				.as_ref()
				.map(|r| r.type_.build(context, namespace, scope))
				.transpose()?,
		})
	}
}

impl<'ast, R: Clone + Ord + Hash> Build<'ast, R> for ast::MapTypeExpr {
	type Target = ty::Struct<R, TypeVariableRef>;

	fn build<D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
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
						f.key.build(context, namespace, scope)?,
						ty::Field {
							attributes: f.metadata.build(context, namespace, scope)?,
							type_: f.value.build(context, namespace, scope)?,
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
		namespace: &BuildingNamespace<'ast>,
		scope: &Scope,
	) -> Result<Self::Target, Error>
	where
		D: RdfContextMut<R>,
	{
		Ok(ty::Enum {
			variants: self
				.variants
				.iter()
				.map(|v| {
					let variant = ty::Variant {
						attributes: v.metadata.build(context, namespace, scope)?,
						type_: v.type_.build(context, namespace, scope)?,
					};

					Ok((v.ident.as_str().to_owned(), variant))
				})
				.collect::<Result<_, _>>()?,
		})
	}
}
