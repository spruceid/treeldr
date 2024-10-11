use iref::{Iri, IriBuf};
use rdf_types::{
	interpretation::{IriInterpretation, ReverseLiteralInterpretation},
	Vocabulary,
};
use static_iref::iri;

use crate::{LayoutRef, LayoutRegistry};

#[cfg(feature = "serde_cbor")]
mod serde_cbor;

/// CBOR extension `tag` property.
///
/// Allows one to specifies the CBOR tag of a TreeLDR layout.
pub const CBOR_TAG_IRI: &Iri = iri!("https://schema.treeldr.org/cbor#tag");

/// Error type returned by the [`get_layout_tag`] function when the value
/// of the [CBOR extension `tag` property](CBOR_TAG_IRI) is invalid.
#[derive(Debug, thiserror::Error)]
pub enum InvalidTag {
	#[error("non literal tag value")]
	NonLiteral,

	#[error("invalid tag value: {0}")]
	Value(String),

	#[error("invalid tag type: {0}")]
	Type(IriBuf),
}

/// Returns the CBOR tag of a given layout (reference).
pub fn get_layout_tag<V, I>(
	vocabulary: &V,
	interpretation: &I,
	layouts: &impl LayoutRegistry<I::Resource>,
	layout_ref: &LayoutRef<I::Resource>,
) -> Result<Option<u64>, InvalidTag>
where
	V: Vocabulary,
	I: IriInterpretation<V::Iri> + ReverseLiteralInterpretation<Literal = V::Literal>,
	I::Resource: Ord,
{
	let layout = layouts.layout(layout_ref).expect("missing layout definition");
	match interpretation.lexical_iri_interpretation(vocabulary, CBOR_TAG_IRI) {
		Some(prop) => {
			match layout.extra_properties().get(&prop) {
				Some(value) => {
					for l in interpretation.literals_of(value) {
						if let Some(literal) = vocabulary.literal(l) {
							if let rdf_types::LiteralTypeRef::Any(ty) = literal.type_ {
								if let Some(ty_iri) = vocabulary.iri(ty) {
									return match xsd_types::UnsignedLongDatatype::from_iri(ty_iri) {
										Some(_) => literal.value.parse().map(Some).map_err(|_| {
											InvalidTag::Value(literal.value.to_owned())
										}),
										None => Err(InvalidTag::Type(ty_iri.to_owned())),
									};
								}
							}
						}
					}

					Err(InvalidTag::NonLiteral)
				}
				None => Ok(None),
			}
		}
		None => Ok(None),
	}
}
