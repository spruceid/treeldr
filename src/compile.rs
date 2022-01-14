use crate::{
	Feature,
	Context,
	Id,
	Error,
	Cause,
	Caused,
	syntax,
	syntax::Loc
};

/// Compilation function.
pub trait Compile {
	type Target;

	fn compile(&self, context: &mut Context) -> Result<Self::Target, Caused<Error>>;
}

impl Compile for Loc<syntax::Id> {
	type Target = Id;

	fn compile(&self, context: &mut Context) -> Result<Self::Target, Caused<Error>> {
		let iri = match self.inner() {
			syntax::Id::IriRef(iri_ref) => {
				iri_ref.resolved(context.base_iri())
			},
			syntax::Id::Compact(_, _) => {
				return Err(Caused::new(Error::Unimplemented(Feature::CompactIri), Some(Cause::Explicit(self.source()))))
			}
		};

		Ok(context.vocabulary_mut().insert(iri))
	}
}

impl Compile for Loc<syntax::Document> {
	type Target = ();

	fn compile(&self, context: &mut Context) -> Result<Self::Target, Caused<Error>> {
		for item in &self.inner().items {
			item.compile(context)?;
		}
		
		Ok(())
	}
}

impl Compile for Loc<syntax::Item> {
	type Target = ();

	fn compile(&self, context: &mut Context) -> Result<Self::Target, Caused<Error>> {
		match self.inner() {
			syntax::Item::Type(ty_def) => {
				ty_def.compile(context)?;
			},
			syntax::Item::Layout(layout_def) => {
				layout_def.compile(context)?;
			}
		}
		
		Ok(())
	}
}

impl Compile for Loc<syntax::TypeDefinition> {
	type Target = ();

	fn compile(&self, context: &mut Context) -> Result<Self::Target, Caused<Error>> {
		let id = self.inner().id.compile(context)?;
		context.declare_type(id, Some(Cause::Explicit(self.source())))?;
		Ok(())
	}
}

impl Compile for Loc<syntax::LayoutDefinition> {
	type Target = ();

	fn compile(&self, context: &mut Context) -> Result<Self::Target, Caused<Error>> {
		let id = self.inner().id.compile(context)?;
		context.declare_layout(id, Some(Cause::Explicit(self.source())))?;
		Ok(())
	}
}