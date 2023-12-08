#[cfg(feature = "derive")]
#[test]
fn id() {
	#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
	#[tldr(id)]
	pub struct Id(rdf_types::Id);

	impl treeldr::AsId for Id {
		fn as_id(&self) -> rdf_types::Id<&iref::Iri, &rdf_types::BlankId> {
			self.0.as_id_ref()
		}
	}

	impl From<rdf_types::Id> for Id {
		fn from(value: rdf_types::Id) -> Self {
			Self(value)
		}
	}
}
