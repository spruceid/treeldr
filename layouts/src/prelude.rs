use std::sync::OnceLock;

use rdf_types::{
	interpretation::{IriInterpretationMut, LiteralInterpretationMut},
	InterpretationMut, VocabularyMut,
};

use crate::{
	abs::{self, Builder},
	layout::LayoutType,
	Layout, LayoutRegistry, Layouts, Ref,
};

const LAYOUTS: [&str; 12] = [
	include_str!("../prelude/unit.json"),
	include_str!("../prelude/boolean.json"),
	include_str!("../prelude/u8.json"),
	include_str!("../prelude/u16.json"),
	include_str!("../prelude/u32.json"),
	include_str!("../prelude/u64.json"),
	include_str!("../prelude/i8.json"),
	include_str!("../prelude/i16.json"),
	include_str!("../prelude/i32.json"),
	include_str!("../prelude/i64.json"),
	include_str!("../prelude/string.json"),
	include_str!("../prelude/id.json"),
];

pub struct Prelude;

impl Prelude {
	pub fn build() -> Layouts {
		let mut builder = Builder::new();
		for json in LAYOUTS {
			let layout: abs::syntax::Layout = serde_json::from_str(json).unwrap();
			layout.build(&mut builder).unwrap();
		}

		builder.build()
	}

	pub fn build_with<V, I>(vocabulary: &mut V, interpretation: &mut I) -> Layouts<I::Resource>
	where
		V: VocabularyMut,
		I: IriInterpretationMut<V::Iri>
			+ LiteralInterpretationMut<V::Literal>
			+ InterpretationMut<V>,
		I::Resource: Clone + Eq + Ord + std::fmt::Debug,
	{
		let mut builder = Builder::<I::Resource>::new();
		for json in LAYOUTS {
			let layout: abs::syntax::Layout = serde_json::from_str(json).unwrap();
			layout
				.build_with_interpretation(vocabulary, interpretation, &mut builder)
				.unwrap();
		}

		builder.build()
	}
}

impl LayoutRegistry for Prelude {
	fn get(&self, id: &Ref<LayoutType>) -> Option<&Layout> {
		static LAYOUTS: OnceLock<Layouts> = OnceLock::new();
		LAYOUTS.get_or_init(Self::build).get(id)
	}
}
