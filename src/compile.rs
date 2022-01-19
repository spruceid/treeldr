use iref::Iri;
use crate::{
	Feature,
	Model,
	Ref,
	Id,
	Error,
	Cause,
	Caused,
	Documentation,
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
	fn id(&self, context: &Model) -> Id {
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
	context: &'c mut Model,
	scope: Option<Scope>
}

impl<'c> Environment<'c> {
	pub fn new(context: &'c mut Model) -> Self {
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

	pub fn ty(&self) -> Option<Ref<ty::Definition>> {
		match self.scope {
			Some(Scope::Type(ty_ref)) => Some(ty_ref),
			_ => None
		}
	}
}

impl<'c> From<&'c mut Model> for Environment<'c> {
	fn from(context: &'c mut Model) -> Self {
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

impl Compile for syntax::Documentation {
	type Target = Documentation;

	fn compile<'c>(&self, _env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let mut short = String::new();
		let mut long = String::new();
		let mut separated = false;
		
		for line in &self.items {
			if separated {
				long.push_str(line);
			} else {
				if line.trim().is_empty() {
					separated = true
				} else {
					short.push_str(line);
				}
			}
		}

		let short = if short.is_empty() { None } else { Some(short) };
		let long = if long.is_empty() { None } else { Some(long) };

		Ok(Documentation::new(short, long))
	}
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

		// Define implicit layouts.
		let mut implicit_layouts = Vec::new();
		for (_, node) in env.context.nodes() {
			if let Some(ty_ref) = node.as_type() {
				if let Some(layout_ref) = node.as_layout() {
					let layout = env.context.layouts().get(layout_ref).unwrap();
					if layout.fields().is_none() && layout.causes().iter().any(|cause| cause.is_implicit()) {
						let ty = env.context.types().get(ty_ref).unwrap();
						implicit_layouts.push((layout_ref, ty.default_fields(env.context)?, ty.causes().map(Cause::into_implicit)))
					}
				}
			}
		}
		for (layout_ref, fields, causes) in implicit_layouts {
			env.context.layouts_mut().get_mut(layout_ref).unwrap().set_fields(fields, causes.preferred());
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
		let ty_ref = env.context.declare_type(id, Some(Cause::Explicit(self.source())));
		let layout_ref = env.context.declare_layout(id, Some(Cause::Implicit(self.source())));

		env.scope = Some(Scope::Type(ty_ref));
		for prop_def in &self.inner().properties {
			prop_def.declare(env)?;
		}
		env.scope = None;

		let doc = self.inner().doc.compile(env)?;
		env.context.types_mut().get_mut(ty_ref).unwrap().set_documentation(doc.clone());
		let doc = self.inner().doc.compile(env)?;
		env.context.layouts_mut().get_mut(layout_ref).unwrap().set_documentation(doc);

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
		let prop_ref = env.context.declare_property(id, Some(Cause::Explicit(self.source())));
		let doc = self.inner().doc.compile(env)?;
		env.context.properties_mut().get_mut(prop_ref).unwrap().set_documentation(doc);
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
			let prop = env.context.properties_mut().get_mut(prop_ref).unwrap();
			prop.declare_type(ty, Some(Cause::Explicit(self.source())))?;
			env.scope = scope
		}

		if let Some(ty_ref) = env.ty() {
			let prop = env.context.properties_mut().get_mut(prop_ref).unwrap();
			prop.declare_domain(ty_ref, Some(Cause::Explicit(self.source())));

			let ty = env.context.types_mut().get_mut(ty_ref).unwrap();
			ty.declare_property(prop_ref, Some(Cause::Explicit(self.source())));
		}

		Ok(())
	}
}

impl Compile for Loc<syntax::TypeExpr> {
	type Target = ty::Expr;

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let ty_id = self.inner().ty.compile(env)?;
		let ty_ref = env.context.require_type(ty_id, Some(self.source()))?;

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
		let layout_ref = env.context.declare_layout(id, Some(Cause::Explicit(self.source())));
		let doc = self.inner().doc.compile(env)?;
		env.context.layouts_mut().get_mut(layout_ref).unwrap().set_documentation(doc);
		Ok(())
	}
}

impl Compile for Loc<syntax::LayoutDefinition> {
	type Target = ();

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let id = self.inner().id.compile(env)?;
		let layout_ref = env.context.get(id).unwrap().as_layout().unwrap();

		let ty_id = self.inner().ty_id.compile(env)?;
		let ty_ref = env.context.require_type(ty_id, Some(self.source()))?;
		env.context.layouts_mut().get_mut(layout_ref).unwrap().declare_type(ty_ref, Some(Cause::Explicit(self.inner().ty_id.source())))?;

		env.scope = Some(Scope::Layout(layout_ref));
		let mut fields = Vec::with_capacity(self.inner().fields.len());
		for field_def in &self.inner().fields {
			fields.push(field_def.compile(env)?);
		}
		env.scope = None;
		env.context.layouts_mut().get_mut(layout_ref).unwrap().declare_fields(fields, Some(Cause::Explicit(self.inner().ty_id.source())))?;
		Ok(())
	}
}

impl Compile for Loc<syntax::FieldDefinition> {
	type Target = layout::Field;

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let id = self.inner().id.compile(env)?;
		let prop_ref = env.context.require_property(id, Some(self.source()))?;

		let name = match self.inner().alias.as_ref() {
			Some(name) => name.inner().as_str().to_owned(),
			None => {
				env.context.vocabulary().get(id).unwrap().path().file_name().expect("invalid property IRI").to_owned()
			}
		};

		let layout_expr = self.inner().layout.compile(env)?;

		Ok(layout::Field::new(prop_ref, name, layout_expr, Some(Cause::Explicit(self.source()))))
	}
}

impl Compile for Loc<syntax::LayoutExpr> {
	type Target = layout::Expr;

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let ty_id = self.inner().layout.compile(env)?;
		let ty_ref = env.context.require_layout(ty_id, Some(Cause::Explicit(self.source())))?;

		let mut args = Vec::with_capacity(self.inner().args.len());

		for arg in &self.inner().args {
			args.push(arg.compile(env)?)
		}

		Ok(layout::Expr::new(ty_ref, args))
	}
}