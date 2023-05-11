use iref::AsIri;
use rdf_types::{Generator, Id, Literal, Object, Triple, VocabularyMut};
use treeldr::vocab;

use crate::LexXrpcParameters;

use super::{build_rdf_list, sub_id, Item, OutputSubject, OutputTriple};

mod body;
mod procedure;
mod query;
mod subscription;

fn process_xrpc_parameters<V: VocabularyMut>(
	vocabulary: &mut V,
	generator: &mut impl Generator<V>,
	stack: &mut Vec<Item<V>>,
	triples: &mut Vec<OutputTriple<V>>,
	id: &OutputSubject<V>,
	parameters: Option<LexXrpcParameters>,
) -> OutputSubject<V>
where
	V::Iri: Clone,
	V::BlankId: Clone,
{
	match parameters {
		Some(params) => build_rdf_list(
			vocabulary,
			generator,
			triples,
			params.properties,
			|vocabulary, generator, triples, (name, p)| {
				let f_id = generator.next(vocabulary);

				triples.push(Triple(
					f_id.clone(),
					vocabulary.insert(vocab::Rdf::Type.as_iri()),
					Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Field.as_iri()))),
				));

				triples.push(Triple(
					f_id.clone(),
					vocabulary.insert(vocab::TreeLdr::Name.as_iri()),
					Object::Literal(Literal::String(name.clone())),
				));

				let item_id = sub_id(vocabulary, id, &name);
				stack.push(Item::XrpcParametersProperty(item_id.clone(), p));

				let t_id = generator.next(vocabulary);
				triples.push(Triple(
					t_id.clone(),
					vocabulary.insert(vocab::Rdf::Type.as_iri()),
					Object::Id(Id::Iri(vocabulary.insert(vocab::TreeLdr::Layout.as_iri()))),
				));

				if params.required.contains(&name) {
					triples.push(Triple(
						t_id.clone(),
						vocabulary.insert(vocab::TreeLdr::Required.as_iri()),
						Object::Id(item_id),
					));
				} else {
					triples.push(Triple(
						t_id.clone(),
						vocabulary.insert(vocab::TreeLdr::Option.as_iri()),
						Object::Id(item_id),
					));
				};

				triples.push(Triple(
					f_id.clone(),
					vocabulary.insert(vocab::TreeLdr::Format.as_iri()),
					Object::Id(t_id),
				));

				Object::Id(f_id)
			},
		),
		None => Id::Iri(vocabulary.insert(vocab::Rdf::Nil.as_iri())),
	}
}
