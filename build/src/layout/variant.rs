use crate::{Context, Error, Node, component::{self, AssertNamed}, resource};
use locspan::Meta;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, Id, IriIndex, Name};

/// Layout variant definition.
#[derive(Clone)]
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
		as_component: &component::Data<M>,
		as_formatted: &component::formatted::Data<M>,
		metadata: M,
	) -> Option<Meta<Name, M>>
	where
		M: Clone,
	{
		if let Id::Iri(term) = as_resource.id {
			if let Ok(Some(name)) = Name::from_iri(vocabulary.iri(&term).unwrap()) {
				return Some(Meta::new(name, metadata));
			}
		}

		if let Some(layout_id) = as_formatted.format.first() {
			if let Some(layout) = context.get(**layout_id).map(Node::as_component) {
				if let Some(name) = layout.name().first() {
					return Some(Meta::new(name.into_value().clone(), metadata))
				}
			}
		}

		None
	}

	fn build<M>(
		&self,
		context: &Context<M>,
		as_resource: &treeldr::node::Data<M>,
		as_component: &treeldr::component::Data<M>,
		as_formatted: &treeldr::component::formatted::Data<M>,
		meta: &M
	) -> Result<treeldr::layout::variant::Definition, Error<M>> where M: Clone {
		as_component.assert_named(as_resource, meta)?;

		Ok(treeldr::layout::variant::Definition)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	Name,
	Format,
}

pub enum BindingRef<'a> {
	Name(&'a Name),
	Format(Id),
}

pub struct Bindings<'a> {
	name: Option<&'a Name>,
	format: Option<Id>,
}

impl<'a> Iterator for Bindings<'a> {
	type Item = BindingRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.name
			.take()
			.map(BindingRef::Name)
			.or_else(|| self.format.take().map(BindingRef::Format))
	}
}
