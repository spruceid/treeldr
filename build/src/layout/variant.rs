use crate::{Context, Error, component::{self, AssertNamed}, resource};
use locspan::Meta;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, Id, IriIndex, Name};

/// Layout variant definition.
#[derive(Debug, Clone)]
pub struct Definition;

impl Definition {
	pub fn new() -> Self {
		Self
	}

	/// Build a default name for this layout variant.
	pub fn default_name<M>(
		&self,
		context: &Context<M>,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		as_resource: &resource::Data<M>,
		as_formatted: &component::formatted::Data<M>
	) -> Option<Meta<Name, M>>
	where
		M: Clone,
	{
		if let Id::Iri(term) = as_resource.id {
			if let Ok(Some(name)) = Name::from_iri(vocabulary.iri(&term).unwrap()) {
				return Some(Meta::new(name, as_resource.metadata.clone()));
			}
		}

		if let Some(layout_id) = as_formatted.format.first() {
			if let Some(layout) = context.get(**layout_id).map(resource::Definition::as_component) {
				if let Some(name) = layout.name().first() {
					return Some(Meta::new(name.into_value().clone(), as_resource.metadata.clone()))
				}
			}
		}

		None
	}

	pub(crate) fn build<M>(
		&self,
		_context: &Context<M>,
		as_resource: &treeldr::node::Data<M>,
		as_component: &treeldr::component::Data<M>,
		_as_formatted: &treeldr::component::formatted::Data<M>,
		meta: M
	) -> Result<Meta<treeldr::layout::variant::Definition, M>, Error<M>> where M: Clone {
		as_component.assert_named(as_resource, &meta)?;

		Ok(Meta(treeldr::layout::variant::Definition, meta))
	}
}