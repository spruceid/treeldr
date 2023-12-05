use std::{collections::HashMap, hash::Hash};

use grdf::BTreeDataset;
use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::{BlankIdBuf, Id, IriVocabulary, ReverseIriInterpretation};
use treeldr_layouts::{
	distill::RdfContext,
	layout::{DataLayout, LayoutType, ListLayout, LiteralLayout},
	Layout, Layouts, Pattern, Ref,
};
use utils::ident_from_iri;

pub mod utils;

pub enum Error {
	MissingTypeIdentifier,
	InvalidFieldIdent(String),
	InvalidVariantIdent(String),
	NoIriRepresentation,
}

pub struct Options<R> {
	idents: HashMap<Ref<LayoutType, R>, syn::Ident>,
}

impl<R> Options<R> {
	pub fn layout_ident<V, I>(
		&self,
		rdf: RdfContext<V, I>,
		layout_ref: &Ref<LayoutType, I::Resource>,
	) -> Result<syn::Ident, Error>
	where
		V: IriVocabulary,
		I: ReverseIriInterpretation<Resource = R, Iri = V::Iri>,
		R: Eq + Hash,
	{
		default_layout_ident(rdf, layout_ref)
			.or_else(|| self.idents.get(layout_ref).cloned())
			.ok_or(Error::MissingTypeIdentifier)
	}
}

pub fn default_layout_ident<V, I>(
	rdf: RdfContext<V, I>,
	layout_ref: &Ref<LayoutType, I::Resource>,
) -> Option<syn::Ident>
where
	V: IriVocabulary,
	I: ReverseIriInterpretation<Iri = V::Iri>,
{
	let mut selected = None;

	for i in rdf.interpretation.iris_of(layout_ref.id()) {
		if let Some(id) = ident_from_iri(rdf.vocabulary.iri(i).unwrap()) {
			if !selected.as_ref().is_some_and(|s| *s < id) {
				selected = Some(id)
			}
		}
	}

	selected
}

pub fn pattern_to_id<V, I>(
	rdf: RdfContext<V, I>,
	pattern: &Pattern<I::Resource>,
) -> Result<Id, Error>
where
	V: IriVocabulary,
	I: ReverseIriInterpretation<Iri = V::Iri>,
{
	match pattern {
		Pattern::Var(i) => Ok(Id::Blank(BlankIdBuf::from_suffix(&i.to_string()).unwrap())),
		Pattern::Resource(r) => rdf
			.interpretation
			.iris_of(r)
			.next()
			.map(|i| Id::Iri(rdf.vocabulary.iri(i).unwrap().to_owned()))
			.ok_or(Error::NoIriRepresentation),
	}
}

pub fn generate_intro_attribute(count: u32, offset: u32) -> TokenStream {
	let names = (offset..(count + offset)).map(|i| i.to_string());

	quote!(intro(#(#names),*))
}

pub fn generate_input_attribute(count: u32) -> TokenStream {
	let names = (0..count).map(|i| i.to_string());

	quote!(input(#(#names),*))
}

pub fn generate_dataset_attribute<V, I>(
	rdf: RdfContext<V, I>,
	dataset: &BTreeDataset<Pattern<I::Resource>>,
) -> Result<TokenStream, Error>
where
	V: IriVocabulary,
	I: ReverseIriInterpretation<Iri = V::Iri>,
{
	let quads = dataset
		.quads()
		.map(|quad| {
			let s = pattern_to_id(rdf, quad.0)?.to_string();
			let p = pattern_to_id(rdf, quad.1)?.to_string();
			let o = pattern_to_id(rdf, quad.2)?.to_string();
			let g = quad
				.3
				.map(|g| Ok(pattern_to_id(rdf, g)?.to_string()))
				.transpose()?;
			Ok(quote!((#s, #p, #o, #g)))
		})
		.collect::<Result<Vec<_>, _>>()?;

	Ok(quote!(dataset(#(#quads),*)))
}

pub fn generate_value_input_attribute<V, I>(
	rdf: RdfContext<V, I>,
	input: &[Pattern<I::Resource>],
) -> Result<TokenStream, Error>
where
	V: IriVocabulary,
	I: ReverseIriInterpretation<Iri = V::Iri>,
{
	let input = input
		.iter()
		.map(|p| Ok(pattern_to_id(rdf, p)?.to_string()))
		.collect::<Result<Vec<_>, _>>()?;

	Ok(quote!(input(#(#input),*)))
}

pub fn generate_value_graph_attribute<V, I>(
	rdf: RdfContext<V, I>,
	graph: &Option<Option<Pattern<I::Resource>>>,
) -> Result<TokenStream, Error>
where
	V: IriVocabulary,
	I: ReverseIriInterpretation<Iri = V::Iri>,
{
	let expr = match graph {
		Some(Some(p)) => {
			let g = pattern_to_id(rdf, p)?.to_string();
			quote!(Some(#g))
		}
		Some(None) => {
			quote!(None)
		}
		None => {
			quote!(_)
		}
	};

	Ok(quote!(graph(#expr)))
}

pub fn generate<V, I>(
	rdf: RdfContext<V, I>,
	layouts: &Layouts<I::Resource>,
	layout_ref: &Ref<LayoutType, I::Resource>,
	options: &Options<I::Resource>,
) -> Result<TokenStream, Error>
where
	V: IriVocabulary,
	I: ReverseIriInterpretation<Iri = V::Iri>,
	I::Resource: Ord + Hash,
{
	let layout = layouts.get(layout_ref).unwrap();
	let ident = options.layout_ident(rdf, layout_ref)?;

	match layout {
		Layout::Always => Ok(quote! {
			pub type #ident = treeldr::Always;
		}),
		Layout::Literal(layout) => match layout {
			LiteralLayout::Data(layout) => match layout {
				DataLayout::Unit(layout) => {
					let input = generate_input_attribute(layout.input);
					let intro = generate_intro_attribute(layout.intro, layout.input);
					let dataset = generate_dataset_attribute(rdf, &layout.dataset)?;

					Ok(quote! {
						#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
						#[tldr(#input, #intro, #dataset)]
						pub struct #ident;
					})
				}
				DataLayout::Boolean(layout) => {
					let input = generate_input_attribute(layout.input);
					let intro = generate_intro_attribute(layout.intro, layout.input);
					let dataset = generate_dataset_attribute(rdf, &layout.dataset)?;

					Ok(quote! {
						#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
						#[tldr(#input, #intro, #dataset)]
						pub struct #ident(bool);
					})
				}
				DataLayout::Number(layout) => {
					let input = generate_input_attribute(layout.input);
					let intro = generate_intro_attribute(layout.intro, layout.input);
					let dataset = generate_dataset_attribute(rdf, &layout.dataset)?;

					Ok(quote! {
						#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
						#[tldr(#input, #intro, #dataset)]
						pub struct #ident(Number);
					})
				}
				DataLayout::TextString(layout) => {
					let input = generate_input_attribute(layout.input);
					let intro = generate_intro_attribute(layout.intro, layout.input);
					let dataset = generate_dataset_attribute(rdf, &layout.dataset)?;

					Ok(quote! {
						#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
						#[tldr(#input, #intro, #dataset)]
						pub struct #ident(String);
					})
				}
				DataLayout::ByteString(layout) => {
					let input = generate_input_attribute(layout.input);
					let intro = generate_intro_attribute(layout.intro, layout.input);
					let dataset = generate_dataset_attribute(rdf, &layout.dataset)?;

					Ok(quote! {
						#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
						#[tldr(#input, #intro, #dataset)]
						pub struct #ident(Vec<u8>);
					})
				}
			},
			LiteralLayout::Id(layout) => {
				let input = generate_input_attribute(layout.input);
				let intro = generate_intro_attribute(layout.intro, layout.input);
				let dataset = generate_dataset_attribute(rdf, &layout.dataset)?;

				Ok(quote! {
					#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
					#[tldr(id, #input, #intro, #dataset)]
					pub struct #ident(Id);
				})
			}
		},
		Layout::Product(layout) => {
			let input = generate_input_attribute(layout.input);
			let intro = generate_intro_attribute(layout.intro, layout.input);
			let dataset = generate_dataset_attribute(rdf, &layout.dataset)?;
			let fields = layout
				.fields
				.iter()
				.map(|(name, f)| {
					let f_ident = syn::parse_str::<syn::Ident>(name.as_str())
						.map_err(|_| Error::InvalidFieldIdent(name.clone()))?;

					let intro = generate_intro_attribute(f.intro, layout.input + layout.intro);
					let dataset = generate_dataset_attribute(rdf, &f.dataset)?;
					let input = generate_value_input_attribute(rdf, &f.value.input)?;
					let graph = generate_value_graph_attribute(rdf, &f.value.graph)?;
					let layout = options.layout_ident(rdf, &f.value.layout)?;

					Ok(quote! {
						#[tldr(#intro, #dataset, #input, #graph)]
						#f_ident : #layout
					})
				})
				.collect::<Result<Vec<_>, _>>()?;

			Ok(quote! {
				#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
				#[tldr(#input, #intro, #dataset)]
				pub struct #ident {
					#(#fields),*
				}
			})
		}
		Layout::Sum(layout) => {
			let input = generate_input_attribute(layout.input);
			let intro = generate_intro_attribute(layout.intro, layout.input);
			let dataset = generate_dataset_attribute(rdf, &layout.dataset)?;
			let variants = layout
				.variants
				.iter()
				.map(|v| {
					let v_ident = syn::parse_str::<syn::Ident>(&v.name)
						.map_err(|_| Error::InvalidVariantIdent(v.name.clone()))?;

					let intro = generate_intro_attribute(v.intro, layout.input + layout.intro);
					let dataset = generate_dataset_attribute(rdf, &v.dataset)?;
					let input = generate_value_input_attribute(rdf, &v.value.input)?;
					let graph = generate_value_graph_attribute(rdf, &v.value.graph)?;
					let layout = options.layout_ident(rdf, &v.value.layout)?;

					Ok(quote! {
						#[tldr(#intro, #dataset, #input, #graph)]
						#v_ident(#layout)
					})
				})
				.collect::<Result<Vec<_>, _>>()?;

			Ok(quote! {
				#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
				#[tldr(#input, #intro, #dataset)]
				pub enum #ident {
					#(#variants),*
				}
			})
		}
		Layout::List(layout) => match layout {
			ListLayout::Unordered(layout) => {
				let input = generate_input_attribute(layout.input);
				let intro = generate_intro_attribute(layout.intro, layout.input);
				let dataset = generate_dataset_attribute(rdf, &layout.dataset)?;

				let item_intro =
					generate_intro_attribute(layout.item.intro, layout.input + layout.intro);
				let item_dataset = generate_dataset_attribute(rdf, &layout.item.dataset)?;
				let item_input = generate_value_input_attribute(rdf, &layout.item.value.input)?;
				let item_graph = generate_value_graph_attribute(rdf, &layout.item.value.graph)?;
				let item_layout = options.layout_ident(rdf, &layout.item.value.layout)?;

				Ok(quote! {
					#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
					#[tldr(set, #intro, #dataset, #input)]
					pub struct #ident(
						#[tldr(#item_intro, #item_dataset, #item_input, #item_graph)]
						::std::collection::BTreeSet<#item_layout>
					);
				})
			}
			ListLayout::Ordered(layout) => {
				let input = generate_input_attribute(layout.input);
				let intro = generate_intro_attribute(layout.intro, layout.input);
				let dataset = generate_dataset_attribute(rdf, &layout.dataset)?;
				let head = pattern_to_id(rdf, &layout.head)?.to_string();
				let tail = pattern_to_id(rdf, &layout.tail)?.to_string();

				let node_intro =
					generate_intro_attribute(layout.node.intro, layout.input + layout.intro + 2);
				let node_head = (layout.input + layout.intro).to_string();
				let node_rest = (layout.input + layout.intro + 1).to_string();
				let node_dataset = generate_dataset_attribute(rdf, &layout.node.dataset)?;
				let node_input = generate_value_input_attribute(rdf, &layout.node.value.input)?;
				let node_graph = generate_value_graph_attribute(rdf, &layout.node.value.graph)?;
				let node_layout = options.layout_ident(rdf, &layout.node.value.layout)?;

				Ok(quote! {
					#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
					#[tldr(list, #intro, head(#head), tail(#tail), #dataset, #input)]
					pub struct #ident(
						#[tldr(head(#node_head), rest(#node_rest), #node_intro, #node_dataset, #node_input, #node_graph)]
						Vec<#node_layout>
					);
				})
			}
			ListLayout::Sized(layout) => {
				let input = generate_input_attribute(layout.input);
				let intro = generate_intro_attribute(layout.intro, layout.input);
				let dataset = generate_dataset_attribute(rdf, &layout.dataset)?;

				let items = layout
					.items
					.iter()
					.map(|item| {
						let item_intro =
							generate_intro_attribute(item.intro, layout.input + layout.intro);
						let item_dataset = generate_dataset_attribute(rdf, &item.dataset)?;
						let item_input = generate_value_input_attribute(rdf, &item.value.input)?;
						let item_graph = generate_value_graph_attribute(rdf, &item.value.graph)?;
						let item_layout = options.layout_ident(rdf, &item.value.layout)?;

						Ok(quote! {
							#[tldr(#item_intro, #item_dataset, #item_input, #item_graph)]
							#item_layout
						})
					})
					.collect::<Result<Vec<_>, _>>()?;

				Ok(quote! {
					#[derive(treeldr::SerializeLd, treeldr::DeserializeLd)]
					#[tldr(tuple, #intro, #dataset, #input)]
					pub struct #ident(#(#items),*);
				})
			}
		},
		Layout::Never => Ok(quote! {
			pub type #ident = treeldr::Never;
		}),
	}
}
