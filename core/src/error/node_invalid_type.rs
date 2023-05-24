use crate::{Id, IriIndex, BlankIdIndex, component, Type, ty::SubClass, prop, node, TId, PropertyValues, PropertyValueRef};
use locspan::{MaybeLocated, Span, Meta};
use rdf_types::Vocabulary;
use contextual::WithContext;

#[derive(Debug)]
pub struct NodeInvalidType<M> {
	pub id: Id,
	pub expected: TId<Type>,
	pub found: PropertyValues<TId<Type>, M>
}

trait NodeTypeName {
	fn name(&self) -> &str;
}

impl NodeTypeName for Type {
	fn name(&self) -> &str {
		match self {
			Self::Resource(None) => "resource",
			Self::Resource(Some(ty)) => ty.name(),
			Self::Other(_) => "unknown"
		}
	}
}

impl NodeTypeName for node::Type {
	fn name(&self) -> &str {
		match self {
			Self::Literal => "literal",
			Self::Class(None) => "class",
			Self::Class(Some(ty)) => ty.name(),
			Self::Property(None) => "property",
			Self::Property(Some(ty)) => ty.name(),
			Self::Component(None) => "component",
			Self::Component(Some(ty)) => ty.name(),
			Self::DatatypeRestriction => "datatype restriction",
			Self::LayoutRestriction => "layout restriction",
			Self::List => "list"
		}
	}
}

impl NodeTypeName for SubClass {
	fn name(&self) -> &str {
		match self {
			Self::DataType => "datatype",
			Self::Restriction => "property restriction"
		}
	}
}

impl NodeTypeName for prop::Type {
	fn name(&self) -> &str {
		match self {
			Self::FunctionalProperty => "functional property"
		}
	}
}

impl NodeTypeName for component::Type {
	fn name(&self) -> &str {
		match self {
			Self::Layout => "layout",
			Self::Formatted(None) => "formatted component",
			Self::Formatted(Some(ty)) => ty.name()
		}
	}
}

impl NodeTypeName for component::formatted::Type {
	fn name(&self) -> &str {
		match self {
			Self::LayoutField => "structure layout field",
			Self::LayoutVariant => "enum layout variant",
		}
	}
}

impl<M: MaybeLocated<Span=Span>> super::AnyError<M> for NodeInvalidType<M> where M::File: Clone {
	fn message(&self, vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> String {
		format!("invalid type for {}", self.id.with(vocab))
	}

	fn labels(&self, _vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> Vec<codespan_reporting::diagnostic::Label<M::File>> {
		let mut labels = Vec::new();

		for PropertyValueRef { value: Meta(ty, metadata), .. } in self.found.iter() {
			let ty: Type = (*ty).into();
			if let Some(loc) = metadata.optional_location().cloned() {
				labels.push(loc.into_secondary_label().with_message(format!("declared as a {} here", ty.name())));
			}
		}

		labels
	}

	fn notes(&self, _vocab: &impl Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>) -> Vec<String> {
		let mut notes = Vec::new();

		let expected: Type = self.expected.into();
		notes.push(format!("expected a {}", expected.name()));

		for (i, ty) in self.found.iter().enumerate() {
			let ty: Type = (**ty.value).into();
			if i == 0 {
				notes.push(format!("found a {}", ty.name()))
			} else {
				notes.push(format!("      a {}", ty.name()))
			}
		}

		notes
	}
}