use crate::{
	component::{self, AssertNamed},
	context::MapIds,
	resource, Context, Error,
};
use locspan::Meta;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, Id, IriIndex, Name};

/// Layout variant definition.
#[derive(Debug, Clone)]
pub struct Definition;

impl Default for Definition {
	fn default() -> Self {
		Self
	}
}

impl Definition {
	pub fn new() -> Self {
		Self::default()
	}

	/// Build a default name for this layout variant.
	///
	/// The default name follows these rules:
	///   - If the variant layout has a name, it is used as default name,
	///   - If the variant layout is a reference, the default name is the
	///     concatenation of the referenced type name and `Ref`.
	pub fn default_name<M>(
		&self,
		context: &Context<M>,
		vocabulary: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		as_resource: &resource::Data<M>,
		as_formatted: &component::formatted::Data<M>,
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
			if let Some(layout) = context.get(**layout_id) {
				if let Some(name) = layout.as_component().name().first() {
					return Some(Meta::new(
						name.into_value().clone(),
						as_resource.metadata.clone(),
					));
				}

				if let Some(Meta(super::Description::Reference(_), _)) =
					layout.as_layout().description().first()
				{
					if let Some(ty_id) = layout.as_layout().ty().first() {
						if let Ok(Some(mut name)) = Name::from_id(vocabulary, **ty_id) {
							name.push("ref");
							return Some(Meta(name, as_resource.metadata.clone()));
						}
					}
				}
			}
		}

		None
	}

	pub fn is_included_in<M>(
		&self,
		context: &Context<M>,
		as_component: &crate::component::Data<M>,
		as_formatted: &crate::component::formatted::Data<M>,
		_other: &Self,
		other_as_component: &crate::component::Data<M>,
		other_as_formatted: &crate::component::formatted::Data<M>,
	) -> bool {
		let common_name = as_component
			.name
			.iter()
			.any(|Meta(a, _)| other_as_component.name.iter().any(|Meta(b, _)| a == b));
		let no_name = as_component.name.is_empty() && other_as_component.name.is_empty();

		let included_layout = as_formatted.format.iter().all(|Meta(a, _)| {
			other_as_formatted
				.format
				.iter()
				.all(|Meta(b, _)| crate::layout::is_included_in(context, *a, *b))
		});

		(common_name || no_name) && included_layout
	}

	pub(crate) fn build<M>(
		&self,
		_context: &Context<M>,
		as_resource: &treeldr::node::Data<M>,
		as_component: &treeldr::component::Data<M>,
		_as_formatted: &treeldr::component::formatted::Data<M>,
		meta: M,
	) -> Result<Meta<treeldr::layout::variant::Definition, M>, Error<M>>
	where
		M: Clone,
	{
		as_component.assert_named(as_resource, &meta)?;

		Ok(Meta(treeldr::layout::variant::Definition, meta))
	}
}

pub fn is_included_in<M>(context: &Context<M>, a: Id, b: Id) -> bool {
	if a == b {
		true
	} else {
		let a = context.get(a).unwrap();
		let b = context.get(b).unwrap();

		a.as_layout_variant().is_included_in(
			context,
			a.as_component().data(),
			a.as_formatted().data(),
			b.as_layout_variant(),
			b.as_component().data(),
			b.as_formatted().data(),
		)
	}
}

impl MapIds for Definition {
	fn map_ids(&mut self, _f: impl Fn(Id, Option<crate::Property>) -> Id) {
		// nothing.
	}
}
