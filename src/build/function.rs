use std::hash::Hash;

use ll1::Spanned;
use rdf_types::InterpretationMut;

use crate::{
	function::{self, Signature},
	syntax::ast,
	Function, ModuleRef, Registry,
};

use super::{
	Build, BuildContext, BuildingPackage, Error, ExprScope, Scope, TypeValue, TypeVariableRef,
};

pub struct FunctionDefinition<'ast, P> {
	pub module: ModuleRef<P>,
	pub metadata: &'ast ast::Metadata,
	pub value: &'ast ast::FunctionDefinition,
	pub signature: BuildSignature,
}

impl<'ast, P: Copy + Ord + Hash> FunctionDefinition<'ast, P> {
	pub fn build<E, D>(
		&self,
		context: &mut BuildContext<E, D>,
		namespace: &BuildingPackage<'ast, P>,
	) -> Result<Function<P, E::Resource, TypeVariableRef>, Error<P>>
	where
		E: ?Sized + Registry<PackageId = P>,
		E::Resource: Clone + Ord + Hash,
		D: InterpretationMut<Resource = E::Resource>,
	{
		let scope = Scope {
			module: Some(self.module),
			expr: ExprScope::new(None),
			const_: false,
		};

		let attributes = self.metadata.build(context, namespace, &scope)?;
		let mut sub_scope = scope.begin();

		let args: Vec<_> = self
			.value
			.args
			.iter()
			.map(|a| {
				Ok(match a {
					ast::ArgDef::Typed(_, ident, ty, _) => {
						let type_ref = ty.expr.build(context, namespace, &scope)?;
						sub_scope.expr.insert(ident, type_ref);
						type_ref
					}
					ast::ArgDef::Untyped(ident) => {
						let type_ref = context
							.type_variables
							.fresh(TypeValue::Undefined, Some(ident.span()));
						sub_scope.expr.insert(ident, type_ref);
						type_ref
					}
				})
			})
			.collect::<Result<_, _>>()?;

		let body = self.value.body.build(context, namespace, &sub_scope)?;

		let return_type = match &self.value.r#type {
			Some(ty) => ty.expr.build(context, namespace, &scope)?,
			None => body.r#type,
		};

		for (&a, &b) in args.iter().zip(&self.signature.args) {
			context.type_variables.assert_eq(a, b);
		}

		context
			.type_variables
			.assert_eq(return_type, self.signature.return_type);

		let dataset = self
			.value
			.where_bound
			.as_ref()
			.map(|b| b.dataset.build(context, namespace, &sub_scope))
			.transpose()?
			.unwrap_or_default();

		Ok(Function {
			attributes,
			signature: Signature::new(
				self.signature.const_args.clone(),
				self.signature.args.clone(),
				self.signature.return_type,
				dataset,
			),
			body: function::Body::Expr(Box::new(body)),
		})
	}
}

#[derive(Debug, Clone)]
pub struct BuildSignature {
	pub const_args: Vec<TypeVariableRef>,
	pub args: Vec<TypeVariableRef>,
	pub return_type: TypeVariableRef,
}

impl BuildSignature {
	pub fn arity(&self) -> usize {
		self.const_args.len() + self.args.len()
	}

	pub fn import<P, R, T>(
		signature: &Signature<P, R, T>,
		mut f: impl FnMut(&T) -> TypeVariableRef,
	) -> Self {
		Self {
			const_args: signature.const_args.iter().map(&mut f).collect(),
			args: signature.args.iter().map(&mut f).collect(),
			return_type: f(&signature.return_type),
		}
	}
}
