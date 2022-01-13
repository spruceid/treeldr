use iref::{Iri, IriBuf};
use std::collections::HashMap;

/// Unique identifier associated to a known IRI.
///
/// This simplifies storage and comparison between IRIs.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Debug)]
pub struct Id(pub(crate) usize);

impl Id {
	pub(crate) fn index(&self) -> usize {
		self.0
	}
}

/// Dictionary storing each known IRI and associated unique `Id`.
#[derive(Default)]
pub struct Vocabulary {
	iri_to_id: HashMap<IriBuf, Id>,
	id_to_iri: Vec<IriBuf>,
}

impl Vocabulary {
	/// Creates a new empty vocabulary.
	pub fn new() -> Self {
		Self::default()
	}

	/// Adds a new IRI to the vocabulary and returns its `Id`.
	///
	/// If the IRI is already in the vocabulary, its `Id` is returned
	/// and the vocabulary is unchanged.
	pub fn insert(&mut self, iri: IriBuf) -> Id {
		use std::collections::hash_map::Entry;
		match self.iri_to_id.entry(iri) {
			Entry::Occupied(entry) => *entry.get(),
			Entry::Vacant(entry) => {
				let id = Id(self.id_to_iri.len());
				self.id_to_iri.push(entry.key().clone());
				entry.insert(id);
				id
			}
		}
	}

	/// Returns the IRI of the given `Id`, if any.
	pub fn get(&self, id: Id) -> Option<Iri> {
		self.id_to_iri.get(id.index()).map(|iri| iri.as_iri())
	}
}
