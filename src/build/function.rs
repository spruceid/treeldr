use std::hash::Hash;

use crate::{
	function::{self, Signature},
	rdf::RdfContextMut,
	syntax::ast,
	Function, ModuleRef, Type,
};

use super::{Build, BuildContext, BuildingNamespace, Error, ExprScope, Scope, TypeVariableRef};

pub struct FunctionDefinition<'ast> {
	pub module: ModuleRef,
	pub metadata: &'ast ast::Metadata,
	pub value: &'ast ast::FunctionDefinition,
	pub signature: BuildSignature,
}

impl<'ast> FunctionDefinition<'ast> {
	pub fn build<R, D>(
		&self,
		context: &mut BuildContext<R, D>,
		namespace: &BuildingNamespace<'ast>,
	) -> Result<Function<R, TypeVariableRef>, Error>
	where
		R: Clone + Ord + Hash,
		D: RdfContextMut<R>,
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
						let type_ref = context.type_variables.fresh(Type::resource());
						sub_scope.expr.insert(ident, type_ref.clone());
						type_ref
					}
				})
			})
			.collect::<Result<_, _>>()?;

		let body = self.value.body.build(context, namespace, &sub_scope)?;

		let return_type = match &self.value.type_ {
			Some(ty) => ty.expr.build(context, namespace, &scope)?,
			None => body.type_,
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
			signature: Signature {
				const_args: self.signature.const_args.clone(),
				args: self.signature.args.clone(),
				return_type: self.signature.return_type,
				dataset,
			},
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

	pub fn import<R, T>(
		signature: &Signature<R, T>,
		mut f: impl FnMut(&T) -> TypeVariableRef,
	) -> Self {
		Self {
			const_args: signature.const_args.iter().map(&mut f).collect(),
			args: signature.args.iter().map(&mut f).collect(),
			return_type: f(&signature.return_type),
		}
	}
}
