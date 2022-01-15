use iref::Iri;
use crate::{
	Feature,
	Context,
	Ref,
	Id,
	Error,
	Cause,
	Caused,
	syntax,
	syntax::Loc,
	ty,
	layout
};

#[derive(Clone, Copy)]
enum Scope {
	Type(Ref<ty::Definition>),
	Layout(Ref<layout::Definition>)
}

impl Scope {
	fn id(&self, context: &Context) -> Id {
		match self {
			Self::Type(ty_ref) => {
				context.types().get(*ty_ref).unwrap().id()
			},
			Self::Layout(layout_ref) => {
				let ty_ref = context.layouts().get(*layout_ref).unwrap().ty().unwrap().reference();
				context.types().get(ty_ref).unwrap().id()
			}
		}
	}
}

/// Compile environment.
pub struct Environment<'c> {
	context: &'c mut Context,
	scope: Option<Scope>
}

impl<'c> Environment<'c> {
	pub fn new(context: &'c mut Context) -> Self {
		Self {
			context,
			scope: None
		}
	}

	pub fn base_iri(&self) -> Iri {
		match &self.scope {
			Some(scope) => {
				let id = scope.id(self.context);
				self.context.vocabulary().get(id).unwrap()
			},
			None => self.context.base_iri()
		}
	}
}

impl<'c> From<&'c mut Context> for Environment<'c> {
	fn from(context: &'c mut Context) -> Self {
		Self::new(context)
	}
}

/// Compilation function.
pub trait Compile {
	type Target;

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>>;
}

pub trait Declare {
	/// Declare types, layouts and properties.
	fn declare<'c>(&self, _env: &mut Environment<'c>) -> Result<(), Caused<Error>>;
}

impl Compile for Loc<syntax::Id> {
	type Target = Id;

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let iri = match self.inner() {
			syntax::Id::IriRef(iri_ref) => {
				iri_ref.resolved(env.base_iri())
			},
			syntax::Id::Compact(_, _) => {
				return Err(Caused::new(Error::Unimplemented(Feature::CompactIri), Some(Cause::Explicit(self.source()))))
			}
		};

		Ok(env.context.vocabulary_mut().insert(iri))
	}
}

impl Compile for Loc<syntax::Document> {
	type Target = ();

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		for item in &self.inner().items {
			item.declare(env)?;
		}

		for item in &self.inner().items {
			item.compile(env)?;
		}
		
		Ok(())
	}
}

impl Declare for Loc<syntax::Item> {
	fn declare<'c>(&self, env: &mut Environment<'c>) -> Result<(), Caused<Error>> {
		match self.inner() {
			syntax::Item::Type(ty_def) => {
				ty_def.declare(env)?;
			},
			syntax::Item::Layout(layout_def) => {
				layout_def.declare(env)?;
			}
		}
		
		Ok(())
	}
}

impl Compile for Loc<syntax::Item> {
	type Target = ();

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		match self.inner() {
			syntax::Item::Type(ty_def) => {
				ty_def.compile(env)?;
			},
			syntax::Item::Layout(layout_def) => {
				layout_def.compile(env)?;
			}
		}
		
		Ok(())
	}
}

impl Declare for Loc<syntax::TypeDefinition> {
	fn declare<'c>(&self, env: &mut Environment<'c>) -> Result<(), Caused<Error>> {
		let id = self.inner().id.compile(env)?;
		let ty_ref = env.context.declare_type(id, Some(Cause::Explicit(self.source())))?;

		env.scope = Some(Scope::Type(ty_ref));
		for prop_def in &self.inner().properties {
			prop_def.declare(env)?;
		}
		env.scope = None;

		Ok(())
	}
}

impl Compile for Loc<syntax::TypeDefinition> {
	type Target = ();

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let id = self.inner().id.compile(env)?;
		let ty_ref = env.context.get(id).unwrap().as_type().unwrap();

		env.scope = Some(Scope::Type(ty_ref));
		for prop_def in &self.inner().properties {
			prop_def.compile(env)?;
		}
		env.scope = None;

		Ok(())
	}
}

impl Declare for Loc<syntax::PropertyDefinition> {
	fn declare<'c>(&self, env: &mut Environment<'c>) -> Result<(), Caused<Error>> {
		let id = self.inner().id.compile(env)?;
		env.context.declare_property(id, Some(Cause::Explicit(self.source())))?;
		Ok(())
	}
}

impl Compile for Loc<syntax::PropertyDefinition> {
	type Target = ();

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let id = self.inner().id.compile(env)?;
		let prop_ref = env.context.get(id).unwrap().as_property().unwrap();

		if let Some(ty_expr) = &self.inner().ty {
			let scope = env.scope.take();
			let ty = ty_expr.compile(env)?;
			env.context.properties_mut().get_mut(prop_ref).unwrap().declare_type(ty, Some(Cause::Explicit(self.source())))?;
			env.scope = scope
		}

		Ok(())
	}
}

impl Compile for Loc<syntax::TypeExpr> {
	type Target = ty::Expr;

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let ty_id = self.inner().ty.compile(env)?;
		let ty_ref = env.context.require_type(ty_id, self.source())?;

		let mut args = Vec::with_capacity(self.inner().args.len());

		for arg in &self.inner().args {
			args.push(arg.compile(env)?)
		}

		Ok(ty::Expr::new(ty_ref, args))
	}
}

impl Declare for Loc<syntax::LayoutDefinition> {
	fn declare<'c>(&self, env: &mut Environment<'c>) -> Result<(), Caused<Error>> {
		let id = self.inner().id.compile(env)?;
		let layout_ref = env.context.declare_layout(id, Some(Cause::Explicit(self.source())))?;
		env.scope = Some(Scope::Layout(layout_ref));
		// TODO
		env.scope = None;
		Ok(())
	}
}

impl Compile for Loc<syntax::LayoutDefinition> {
	type Target = ();

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let id = self.inner().id.compile(env)?;
		let layout_ref = env.context.get(id).unwrap().as_layout().unwrap();

		let ty_id = self.inner().ty_id.compile(env)?;
		let ty_ref = env.context.require_type(ty_id, self.source())?;
		env.context.layouts_mut().get_mut(layout_ref).unwrap().declare_type(ty_ref, Some(Cause::Explicit(self.inner().ty_id.source())))?;

		env.scope = Some(Scope::Layout(layout_ref));
		// TODO
		env.scope = None;
		Ok(())
	}
}