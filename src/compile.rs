use crate::{
	layout, syntax,
	syntax::{Annotation, Loc},
	ty, Cause, Caused, Documentation, Error, Id, Model, Ref,
};
use iref::{IriBuf, IriRef};
use std::collections::HashMap;

#[derive(Clone, Copy)]
enum Scope {
	Type(Ref<ty::Definition>),
	Layout(Ref<layout::Definition>),
}

impl Scope {
	fn id(&self, context: &Model) -> Id {
		match self {
			Self::Type(ty_ref) => context.types().get(*ty_ref).unwrap().id(),
			Self::Layout(layout_ref) => {
				let ty_ref = *context
					.layouts()
					.get(*layout_ref)
					.unwrap()
					.ty()
					.unwrap()
					.inner();
				context.types().get(ty_ref).unwrap().id()
			}
		}
	}
}

/// Compile environment.
pub struct Environment<'c> {
	context: &'c mut Model,
	scope: Option<Scope>,
	aliases: HashMap<String, Caused<IriBuf>>,
}

impl<'c> Environment<'c> {
	pub fn new(context: &'c mut Model) -> Self {
		Self {
			context,
			scope: None,
			aliases: HashMap::new(),
		}
	}

	pub fn base_iri(&self) -> IriBuf {
		match &self.scope {
			Some(scope) => {
				let id = scope.id(self.context);
				let mut iri = self.context.vocabulary().get(id).unwrap().to_owned();
				iri.path_mut().open();
				iri
			}
			None => self.context.base_iri().to_owned(),
		}
	}

	pub fn ty(&self) -> Option<Ref<ty::Definition>> {
		match self.scope {
			Some(Scope::Type(ty_ref)) => Some(ty_ref),
			_ => None,
		}
	}

	pub fn import(
		&mut self,
		prefix: String,
		iri: IriBuf,
		cause: Option<Cause>,
	) -> Result<(), Caused<Error>> {
		use std::collections::hash_map::Entry;
		match self.aliases.entry(prefix) {
			Entry::Vacant(entry) => {
				entry.insert(Caused::new(iri, cause));
				Ok(())
			}
			Entry::Occupied(entry) => Err(Caused::new(
				Error::PrefixRedefinition(entry.key().clone(), entry.get().cause()),
				cause,
			)),
		}
	}

	pub fn expand_compact_iri(
		&self,
		prefix: &str,
		iri_ref: IriRef,
		cause: Option<Cause>,
	) -> Result<IriBuf, Caused<Error>> {
		match self.aliases.get(prefix) {
			Some(iri) => match IriBuf::try_from(iri.as_str().to_string() + iri_ref.as_str()) {
				Ok(iri) => Ok(iri),
				Err((_, string)) => {
					Err(Caused::new(Error::InvalidExpandedCompactIri(string), cause))
				}
			},
			None => Err(Caused::new(
				Error::UndefinedPrefix(prefix.to_owned()),
				cause,
			)),
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
			} else if line.trim().is_empty() {
				separated = true
			} else {
				short.push_str(line);
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
		let iri = match self.value() {
			syntax::Id::Name(name) => IriRef::new(name).unwrap().resolved(env.base_iri().as_iri()),
			syntax::Id::IriRef(iri_ref) => iri_ref.resolved(env.base_iri().as_iri()),
			syntax::Id::Compact(prefix, iri_ref) => env.expand_compact_iri(
				prefix,
				iri_ref.as_iri_ref(),
				Some(Cause::Explicit(*self.location())),
			)?,
		};

		Ok(env.context.vocabulary_mut().insert(iri))
	}
}

impl Compile for Loc<syntax::Document> {
	type Target = ();

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		for import in &self.value().imports {
			import.declare(env)?;
		}

		for ty in &self.value().types {
			ty.declare(env)?;
		}

		for layout in &self.value().layouts {
			layout.declare(env)?;
		}

		for ty in &self.value().types {
			ty.compile(env)?;
		}

		for layout in &self.value().layouts {
			layout.compile(env)?;
		}

		// Define implicit layouts.
		let mut implicit_layouts = Vec::new();
		for (_, node) in env.context.nodes() {
			if let Some(ty_ref) = node.as_type() {
				if let Some(layout_ref) = node.as_layout() {
					let layout = env.context.layouts().get(layout_ref).unwrap();
					if layout.description().is_none()
						&& layout.causes().iter().any(|cause| cause.is_implicit())
					{
						let ty = env.context.types().get(ty_ref).unwrap();
						implicit_layouts.push((
							layout_ref,
							ty.default_fields(env.context)?,
							ty.causes().map(Cause::into_implicit),
						))
					}
				}
			}
		}
		for (layout_ref, fields, causes) in implicit_layouts {
			env.context
				.layouts_mut()
				.get_mut(layout_ref)
				.unwrap()
				.set_fields(fields, causes.preferred());
		}

		env.context.check()
	}
}

impl Declare for Loc<syntax::Import> {
	fn declare<'c>(&self, env: &mut Environment<'c>) -> Result<(), Caused<Error>> {
		env.import(
			self.prefix.as_str().to_owned(),
			self.iri.value().clone(),
			Some(Cause::Explicit(*self.location())),
		)
	}
}

impl Compile for Loc<syntax::Item> {
	type Target = ();

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		match self.value() {
			syntax::Item::Import(_) => (),
			syntax::Item::Type(ty_def) => {
				ty_def.compile(env)?;
			}
			syntax::Item::Layout(layout_def) => {
				layout_def.compile(env)?;
			}
		}

		Ok(())
	}
}

impl Declare for Loc<syntax::TypeDefinition> {
	fn declare<'c>(&self, env: &mut Environment<'c>) -> Result<(), Caused<Error>> {
		let id = self.value().id.compile(env)?;
		let ty_ref = env
			.context
			.declare_type(id, Some(Cause::Explicit(*self.location())));
		let layout_ref = env
			.context
			.declare_layout(id, Some(Cause::Implicit(*self.location())));

		env.scope = Some(Scope::Type(ty_ref));
		for prop_def in &self.value().properties {
			prop_def.declare(env)?;
		}
		env.scope = None;

		let doc = self.value().doc.compile(env)?;
		env.context
			.types_mut()
			.get_mut(ty_ref)
			.unwrap()
			.set_documentation(doc);
		let doc = self.value().doc.compile(env)?;
		let layout = env.context.layouts_mut().get_mut(layout_ref).unwrap();
		layout.declare_type(ty_ref, Some(Cause::Implicit(*self.location())))?;
		layout.set_documentation(doc);

		Ok(())
	}
}

impl Compile for Loc<syntax::TypeDefinition> {
	type Target = ();

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let id = self.value().id.compile(env)?;
		let ty_ref = env.context.get(id).unwrap().as_type().unwrap();

		env.scope = Some(Scope::Type(ty_ref));
		for prop_def in &self.value().properties {
			prop_def.compile(env)?;
		}
		env.scope = None;

		Ok(())
	}
}

impl Declare for Loc<syntax::PropertyDefinition> {
	fn declare<'c>(&self, env: &mut Environment<'c>) -> Result<(), Caused<Error>> {
		let id = self.value().id.compile(env)?;
		let prop_ref = env
			.context
			.declare_property(id, Some(Cause::Explicit(*self.location())));
		let doc = self.value().doc.compile(env)?;
		env.context
			.properties_mut()
			.get_mut(prop_ref)
			.unwrap()
			.set_documentation(doc);
		Ok(())
	}
}

impl Compile for Loc<syntax::PropertyDefinition> {
	type Target = ();

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let id = self.value().id.compile(env)?;
		let prop_ref = env.context.get(id).unwrap().as_property().unwrap();

		if let Some(annotated_ty_expr) = &self.value().ty {
			let ty = annotated_ty_expr.expr.compile(env)?;
			let prop = env.context.properties_mut().get_mut(prop_ref).unwrap();
			prop.declare_type(ty, Some(Cause::Explicit(*self.location())))?;

			for a in &annotated_ty_expr.annotations {
				match a.value() {
					Annotation::Required => prop.declare_required(),
					Annotation::Single => prop.declare_functional(),
				}
			}
		}

		if let Some(ty_ref) = env.ty() {
			let prop = env.context.properties_mut().get_mut(prop_ref).unwrap();
			prop.declare_domain(ty_ref, Some(Cause::Explicit(*self.location())));

			let ty = env.context.types_mut().get_mut(ty_ref).unwrap();
			ty.declare_property(prop_ref, Some(Cause::Explicit(*self.location())));
		}

		Ok(())
	}
}

impl Compile for Loc<syntax::TypeExpr> {
	type Target = ty::Expr;

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let scope = env.scope.take();
		let ty_id = self.value().ty.compile(env)?;
		let ty_ref = env.context.require_type(ty_id, Some(*self.location()))?;

		let mut args = Vec::with_capacity(self.value().args.len());

		for arg in &self.value().args {
			args.push(arg.compile(env)?)
		}

		env.scope = scope;
		Ok(ty::Expr::new(ty_ref, args))
	}
}

impl Declare for Loc<syntax::LayoutDefinition> {
	fn declare<'c>(&self, env: &mut Environment<'c>) -> Result<(), Caused<Error>> {
		let id = self.value().id.compile(env)?;
		let layout_ref = env
			.context
			.declare_layout(id, Some(Cause::Explicit(*self.location())));
		let doc = self.value().doc.compile(env)?;
		env.context
			.layouts_mut()
			.get_mut(layout_ref)
			.unwrap()
			.set_documentation(doc);
		Ok(())
	}
}

impl Compile for Loc<syntax::LayoutDefinition> {
	type Target = ();

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let id = self.value().id.compile(env)?;
		let layout_ref = env.context.get(id).unwrap().as_layout().unwrap();

		let ty_id = self.value().ty_id.compile(env)?;
		let ty_ref = env.context.require_type(ty_id, Some(*self.location()))?;
		env.context
			.layouts_mut()
			.get_mut(layout_ref)
			.unwrap()
			.declare_type(
				ty_ref,
				Some(Cause::Explicit(*self.value().ty_id.location())),
			)?;

		env.scope = Some(Scope::Layout(layout_ref));
		let mut fields = Vec::with_capacity(self.value().fields.len());
		for field_def in &self.value().fields {
			fields.push(field_def.compile(env)?);
		}
		env.scope = None;
		env.context
			.layouts_mut()
			.get_mut(layout_ref)
			.unwrap()
			.declare_fields(fields, Some(Cause::Explicit(*self.value().id.location())))?;
		Ok(())
	}
}

impl Compile for Loc<syntax::FieldDefinition> {
	type Target = layout::Field;

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let id = self.value().id.compile(env)?;
		let prop_ref = env.context.require_property(id, Some(*self.location()))?;

		let name = match self.value().alias.as_ref() {
			Some(name) => name.value().as_str().to_owned(),
			None => env
				.context
				.vocabulary()
				.get(id)
				.unwrap()
				.path()
				.file_name()
				.expect("invalid property IRI")
				.to_owned(),
		};

		let layout_expr = self.value().layout.expr.compile(env)?;
		let mut field = layout::Field::new(
			prop_ref,
			name,
			layout_expr,
			Some(Cause::Explicit(*self.location())),
		);

		for a in &self.value().layout.annotations {
			match a.value() {
				Annotation::Required => field.declare_required(),
				Annotation::Single => field.declare_functional(),
			}
		}

		Ok(field)
	}
}

impl Compile for Loc<syntax::LayoutExpr> {
	type Target = layout::Expr;

	fn compile<'c>(&self, env: &mut Environment<'c>) -> Result<Self::Target, Caused<Error>> {
		let scope = env.scope.take();

		let ty_id = self.value().layout.compile(env)?;
		let ty_ref = env
			.context
			.require_layout(ty_id, Some(Cause::Explicit(*self.location())))?;

		let mut args = Vec::with_capacity(self.value().args.len());

		for arg in &self.value().args {
			args.push(arg.compile(env)?)
		}

		env.scope = scope;
		Ok(layout::Expr::new(ty_ref, args))
	}
}
