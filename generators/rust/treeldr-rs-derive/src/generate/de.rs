use proc_macro2::{Span, TokenStream};
use quote::quote;
use rdf_types::{Id, Term};
use syn::DeriveInput;
use treeldr_layouts::{
	layout::{DataLayout, ListLayout, LiteralLayout},
	Dataset, Layout, Pattern,
};

use crate::parse::parse;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	Parse(#[from] crate::parse::Error),

	#[error(transparent)]
	Build(#[from] treeldr_layouts::abs::syntax::Error),

	#[error("invalid datatype `{0}`")]
	InvalidDatatype(String),
}

impl Error {
	pub fn span(&self) -> Span {
		match self {
			Self::Parse(e) => e.span(),
			Self::Build(_) => Span::call_site(),
			Self::InvalidDatatype(_) => Span::call_site(),
		}
	}
}

pub fn generate(input: DeriveInput) -> Result<TokenStream, Error> {
	let input = parse(input)?;

	let ident = input.ident;

	let mut builder = treeldr_layouts::abs::Builder::new();
	let layout_ref = input.layout.build(&mut builder)?;
	let layouts = builder.build();

	let layout = layouts.get(&layout_ref).unwrap();
	let n = layout.input_count().unwrap() as usize;

	let mut extra: Option<TokenStream> = None;

	let body = match layout {
		Layout::Always => {
			unreachable!()
		}
		Layout::Literal(layout) => match layout {
			LiteralLayout::Id(layout) => {
				let intro = layout.intro;
				let dataset = dataset_to_array(&layout.dataset);
				let resource = generate_pattern(&layout.resource);

				quote! {
					let mut substitution = ::treeldr::pattern::Substitution::from_inputs(inputs);
					substitution.intro(#intro);

					let substitution = ::treeldr::de::Matching::new(
						dataset,
						substitution.clone(),
						::treeldr::utils::QuadsExt::with_default_graph(
							#dataset
								.ok_or(::treeldr::DeserializeError::MissingData)?
								.iter()
								.map(::treeldr::rdf_types::Quad::borrow_components),
							current_graph
						),
					)
					.into_required_unique()?;

					let resource = #resource.ok_or(::treeldr::DeserializeError::MissingData)?.apply(&substitution).into_resource().unwrap();

					let mut selected = None;

					for i in rdf.interpretation.iris_of(&resource) {
						let iri = rdf.vocabulary.iri(i).unwrap();

						// TODO check automaton.

						if selected.replace(::treeldr::rdf_types::Id::Iri(iri.to_owned())).is_some() {
							return Err(::treeldr::DeserializeError::AmbiguousId)
						}
					}

					match selected {
						Some(id) => Ok(Self(::std::convert::TryFrom::try_from(id).map_err(|_| ::treeldr::DeserializeError::InvalidId)?)),
						None => {
							return Err(::treeldr::DeserializeError::MissingId)
						}
					}
				}
			}
			LiteralLayout::Data(layout) => match layout {
				DataLayout::Unit(layout) => {
					let intro = layout.intro;
					let dataset = dataset_to_array(&layout.dataset);

					quote! {
						let mut substitution = ::treeldr::pattern::Substitution::from_inputs(inputs);
						substitution.intro(#intro);

						let substitution = ::treeldr::de::Matching::new(
							dataset,
							substitution.clone(),
							::treeldr::utils::QuadsExt::with_default_graph(
								#dataset
									.ok_or(::treeldr::DeserializeError::MissingData)?
									.iter()
									.map(::treeldr::rdf_types::Quad::borrow_components),
								current_graph
							),
						)
						.into_required_unique()?;

						Ok(Self)
					}
				}
				DataLayout::Boolean(layout) => generate_data(
					&ident,
					&mut extra,
					layout.intro,
					&layout.dataset,
					&layout.resource,
					&layout.datatype,
				)?,
				DataLayout::Number(layout) => generate_data(
					&ident,
					&mut extra,
					layout.intro,
					&layout.dataset,
					&layout.resource,
					&layout.datatype,
				)?,
				DataLayout::ByteString(layout) => generate_data(
					&ident,
					&mut extra,
					layout.intro,
					&layout.dataset,
					&layout.resource,
					&layout.datatype,
				)?,
				DataLayout::TextString(layout) => generate_data(
					&ident,
					&mut extra,
					layout.intro,
					&layout.dataset,
					&layout.resource,
					&layout.datatype,
				)?,
			},
		},
		Layout::Product(layout) => {
			let intro = layout.intro;
			let dataset = dataset_to_array(&layout.dataset);

			let opt_fields = layout.fields.iter().map(|(name, field)| {
				let ident = syn::Ident::new(name, Span::call_site());
				let ty = input
					.type_map
					.get(field.value.layout.id().as_iri().unwrap())
					.unwrap();
				quote!(#ident : Option<#ty>)
			});

			let deserialize_fields = layout.fields.iter().map(|(name, field)| {
				let field_ident = syn::Ident::new(name, Span::call_site());
				let field_ty = input
					.type_map
					.get(field.value.layout.id().as_iri().unwrap())
					.unwrap();
				let field_intro = field.intro;
				let field_dataset = dataset_to_array(&field.dataset);
				let field_inputs = inputs_to_array(&field.value.input);
				let field_graph = generate_graph_pattern(&field.value.graph);

				let m = field.value.input.len();

				quote! {
					let mut field_substitution = substitution.clone();
					field_substitution.intro(#field_intro);

					let field_substitution = ::treeldr::de::Matching::new(
						dataset,
						field_substitution,
						::treeldr::utils::QuadsExt::with_default_graph(
							#field_dataset
								.ok_or(::treeldr::DeserializeError::MissingData)?
								.iter()
								.map(::treeldr::rdf_types::Quad::borrow_components),
							current_graph
						),
					)
					.into_unique()?;

					if let Some(field_substitution) = field_substitution {
						let field_inputs = ::treeldr::de::select_inputs(&#field_inputs.ok_or(::treeldr::DeserializeError::MissingData)?, &field_substitution);

						let item_graph =
							::treeldr::de::select_graph(current_graph, &#field_graph.ok_or(::treeldr::DeserializeError::MissingData)?, &field_substitution);

						let value = <#field_ty as ::treeldr::DeserializeLd<#m, V, I>>::deserialize_ld_with(
							rdf,
							dataset,
							item_graph.as_ref(),
							&field_inputs
						)?;

						data.#field_ident = Some(value);
					}
				}
			});

			let unwrap_fields = layout.fields.keys().map(|name| {
				let ident = syn::Ident::new(name, Span::call_site());
				quote!(#ident: data.#ident.ok_or_else(|| ::treeldr::DeserializeError::MissingField(#name.to_owned()))?)
			});

			quote! {
				let mut substitution = ::treeldr::pattern::Substitution::from_inputs(inputs);
				substitution.intro(#intro);

				let substitution = ::treeldr::de::Matching::new(
					dataset,
					substitution.clone(),
					::treeldr::utils::QuadsExt::with_default_graph(
						#dataset
							.ok_or(::treeldr::DeserializeError::MissingData)?
							.iter()
							.map(::treeldr::rdf_types::Quad::borrow_components),
						current_graph
					),
				)
				.into_required_unique()?;

				#[derive(Default)]
				struct Data {
					#(#opt_fields),*
				}

				let mut data = Data::default();

				#(#deserialize_fields)*

				Ok(Self {
					#(#unwrap_fields)*
				})
			}
		}
		Layout::Sum(layout) => {
			let intro = layout.intro;
			let dataset = dataset_to_array(&layout.dataset);

			let variants = layout.variants.iter().map(|variant| {
				let variant_ident = syn::Ident::new(&variant.name, Span::call_site());
				let variant_ty = input
					.type_map
					.get(variant.value.layout.id().as_iri().unwrap())
					.unwrap();
				let variant_intro = variant.intro;
				let variant_dataset = dataset_to_array(&variant.dataset);
				let variant_inputs = inputs_to_array(&variant.value.input);
				let variant_graph = generate_graph_pattern(&variant.value.graph);
				let m = variant.value.input.len();

				quote! {
					let mut variant_substitution = substitution.clone();
					variant_substitution.intro(#variant_intro);

					let variant_substitution = ::treeldr::de::Matching::new(
						dataset,
						variant_substitution,
						::treeldr::utils::QuadsExt::with_default_graph(
							#variant_dataset
								.ok_or(::treeldr::DeserializeError::MissingData)?
								.iter()
								.map(::treeldr::rdf_types::Quad::borrow_components),
							current_graph
						),
					)
					.into_unique()?;

					if let Some(variant_substitution) = variant_substitution {
						let variant_inputs = ::treeldr::de::select_inputs(&#variant_inputs.ok_or(::treeldr::DeserializeError::MissingData)?, &variant_substitution);

						let variant_graph =
							::treeldr::de::select_graph(current_graph, &#variant_graph.ok_or(::treeldr::DeserializeError::MissingData)?, &variant_substitution);

						let value = <#variant_ty as ::treeldr::DeserializeLd<#m, V, I>>::deserialize_ld_with(
							rdf,
							dataset,
							variant_graph.as_ref(),
							&variant_inputs
						)?;

						if let Some(other) = result.replace(Self::#variant_ident(value)) {
							return Err(::treeldr::DeserializeError::DataAmbiguity)
						}
					}
				}
			});

			quote! {
				let mut substitution = ::treeldr::pattern::Substitution::from_inputs(inputs);
				substitution.intro(#intro);

				let substitution = ::treeldr::de::Matching::new(
					dataset,
					substitution.clone(),
					::treeldr::utils::QuadsExt::with_default_graph(
						#dataset
							.ok_or(::treeldr::DeserializeError::MissingData)?
							.iter()
							.map(::treeldr::rdf_types::Quad::borrow_components),
						current_graph
					),
				)
				.into_required_unique()?;

				let mut result = None;
				#(#variants)*

				result.ok_or(::treeldr::DeserializeError::MissingData)
			}
		}
		Layout::List(layout) => match layout {
			ListLayout::Unordered(layout) => {
				let intro = layout.intro;
				let dataset = dataset_to_array(&layout.dataset);

				let node_intro = layout.item.intro;
				let node_dataset = dataset_to_array(&layout.item.dataset);
				let node_inputs = inputs_to_array(&layout.item.value.input);
				let node_graph = generate_graph_pattern(&layout.item.value.graph);

				let node_ty = input
					.type_map
					.get(layout.item.value.layout.id().as_iri().unwrap())
					.unwrap();

				let m = layout.item.value.input.len();

				quote! {
					let mut substitution = ::treeldr::pattern::Substitution::from_inputs(inputs);
					substitution.intro(#intro);

					let mut substitution = ::treeldr::de::Matching::new(
						dataset,
						substitution,
						::treeldr::utils::QuadsExt::with_default_graph(
							#dataset
								.ok_or(::treeldr::DeserializeError::MissingData)?
								.iter()
								.map(::treeldr::rdf_types::Quad::borrow_components),
							current_graph
						),
					)
					.into_required_unique()?;

					let mut items = Vec::new();

					substitution.intro(#node_intro);
					let matching_dataset = #node_dataset
						.ok_or(::treeldr::DeserializeError::MissingData)?;
					let matching = ::treeldr::de::Matching::new(
						dataset,
						substitution,
						::treeldr::utils::QuadsExt::with_default_graph(
							matching_dataset
								.iter()
								.map(::treeldr::rdf_types::Quad::borrow_components),
							current_graph
						)
					);

					for item_substitution in matching {
						let item_inputs =
							::treeldr::de::select_inputs(&#node_inputs.ok_or(::treeldr::DeserializeError::MissingData)?, &item_substitution);

						let item_graph = ::treeldr::de::select_graph(
							current_graph,
							&#node_graph.ok_or(::treeldr::DeserializeError::MissingData)?,
							&item_substitution,
						);

						let item = <#node_ty as ::treeldr::DeserializeLd<#m, V, I>>::deserialize_ld_with(
							rdf,
							dataset,
							item_graph.as_ref(),
							&item_inputs
						)?;

						items.push(item);
					}

					Ok(Self(items))
				}
			}
			ListLayout::Ordered(layout) => {
				let intro = layout.intro;
				let dataset = dataset_to_array(&layout.dataset);

				let head = generate_pattern(&layout.head);
				let tail = generate_pattern(&layout.tail);

				let node_intro = layout.node.intro;
				let node_dataset = dataset_to_array(&layout.node.dataset);
				let node_inputs = inputs_to_array(&layout.node.value.input);
				let node_graph = generate_graph_pattern(&layout.node.value.graph);

				let node_ty = input
					.type_map
					.get(layout.node.value.layout.id().as_iri().unwrap())
					.unwrap();

				let m = layout.node.value.input.len();

				quote! {
					let mut substitution = ::treeldr::pattern::Substitution::from_inputs(inputs);
					substitution.intro(#intro);

					let substitution = ::treeldr::de::Matching::new(
						dataset,
						substitution.clone(),
						::treeldr::utils::QuadsExt::with_default_graph(
							#dataset
								.ok_or(::treeldr::DeserializeError::MissingData)?
								.iter()
								.map(::treeldr::rdf_types::Quad::borrow_components),
							current_graph
						),
					)
					.into_required_unique()?;

					let mut head = #head.ok_or(::treeldr::DeserializeError::MissingData)?.apply(&substitution).into_resource().unwrap();
					let tail = #tail.ok_or(::treeldr::DeserializeError::MissingData)?.apply(&substitution).into_resource().unwrap();
					let mut items = Vec::new();

					while head != tail {
						let mut item_substitution = substitution.clone();
						item_substitution.push(Some(head));
						let rest = item_substitution.intro(1 + #node_intro);

						let item_substitution = ::treeldr::de::Matching::new(
							dataset,
							item_substitution,
							::treeldr::utils::QuadsExt::with_default_graph(
								#node_dataset
									.ok_or(::treeldr::DeserializeError::MissingData)?
									.iter()
									.map(::treeldr::rdf_types::Quad::borrow_components),
								current_graph
							)
						)
						.into_required_unique()?;

						let item_inputs =
							::treeldr::de::select_inputs(&#node_inputs.ok_or(::treeldr::DeserializeError::MissingData)?, &item_substitution);

						let item_graph = ::treeldr::de::select_graph(
							current_graph,
							&#node_graph.ok_or(::treeldr::DeserializeError::MissingData)?,
							&item_substitution,
						);

						let item = <#node_ty as ::treeldr::DeserializeLd<#m, V, I>>::deserialize_ld_with(
							rdf,
							dataset,
							item_graph.as_ref(),
							&item_inputs
						)?;

						items.push(item);

						head = item_substitution.get(rest).unwrap().clone();
					}

					Ok(Self(items))
				}
			}
			ListLayout::Sized(layout) => {
				let intro = layout.intro;
				let dataset = dataset_to_array(&layout.dataset);

				let init_items = (0..layout.items.len()).map(|_| quote!(None));

				let items = layout.items.iter().enumerate().map(|(i, item)| {
					let index: syn::Index = i.into();
					let node_intro = item.intro;
					let node_dataset = dataset_to_array(&item.dataset);
					let node_inputs = inputs_to_array(&item.value.input);
					let node_graph = generate_graph_pattern(&item.value.graph);

					let node_ty = input
						.type_map
						.get(item.value.layout.id().as_iri().unwrap())
						.unwrap();
					let m = item.value.input.len();
					quote!{
						let mut item_substitution = substitution.clone();
						item_substitution.intro(#node_intro);
						let item_substitution = ::treeldr::de::Matching::new(
							dataset,
							item_substitution,
							::treeldr::utils::QuadsExt::with_default_graph(
								#node_dataset
									.ok_or(::treeldr::DeserializeError::MissingData)?
									.iter()
									.map(::treeldr::rdf_types::Quad::borrow_components),
								current_graph
							)
						).into_required_unique()?;

						let item_inputs =
							::treeldr::de::select_inputs(&#node_inputs.ok_or(::treeldr::DeserializeError::MissingData)?, &item_substitution);

						let item_graph = ::treeldr::de::select_graph(
							current_graph,
							&#node_graph.ok_or(::treeldr::DeserializeError::MissingData)?,
							&item_substitution,
						);

						let item = <#node_ty as ::treeldr::DeserializeLd<#m, V, I>>::deserialize_ld_with(
							rdf,
							dataset,
							item_graph.as_ref(),
							&item_inputs
						)?;

						result.#index = Some(item);
					}
				});

				let unwrap_items = (0..layout.items.len()).map(|i| {
					let index: syn::Index = i.into();
					quote!(result.#index.unwrap())
				});

				quote! {
					let mut substitution = ::treeldr::pattern::Substitution::from_inputs(inputs);
					substitution.intro(#intro);

					let mut substitution = ::treeldr::de::Matching::new(
						dataset,
						substitution,
						::treeldr::utils::QuadsExt::with_default_graph(
							#dataset
								.ok_or(::treeldr::DeserializeError::MissingData)?
								.iter()
								.map(::treeldr::rdf_types::Quad::borrow_components),
							current_graph
						),
					)
					.into_required_unique()?;

					let mut result = (#(#init_items),*);

					#(#items)*

					Ok(Self(#(#unwrap_items),*))
				}
			}
		},
		Layout::Never => {
			unreachable!()
		}
	};

	Ok(quote! {
		impl<V, I> ::treeldr::DeserializeLd<#n, V, I> for #ident
		where
			V: ::treeldr::rdf_types::Vocabulary<Value = String, Type = ::treeldr::RdfType<V>>,
			I: ::treeldr::rdf_types::TermInterpretation<V::Iri, V::BlankId, V::Literal> + ::treeldr::rdf_types::ReverseTermInterpretation<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>,
			I::Resource: Clone + Ord
		{
			fn deserialize_ld_with<D>(
				rdf: ::treeldr::RdfContext<V, I>,
				dataset: &D,
				current_graph: Option<&I::Resource>,
				inputs: &[I::Resource; #n],
			) -> Result<Self, ::treeldr::DeserializeError>
			where
				D: ::treeldr::grdf::Dataset<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource, GraphLabel = I::Resource>
			{
				#body
			}
		}

		#extra
	})
}

fn generate_graph_pattern(graph: &Option<Option<Pattern<Term>>>) -> TokenStream {
	match graph {
		Some(Some(g)) => {
			let g = generate_pattern(g);
			quote!(#g.map(|g| Some(Some(g))))
		}
		Some(None) => quote!(Some(Some(None))),
		None => quote!(Some(None)),
	}
}

fn dataset_to_array(dataset: &Dataset) -> TokenStream {
	let quads = dataset.quads().map(|q| {
		let s = generate_pattern(q.0);
		let p = generate_pattern(q.1);
		let o = generate_pattern(q.2);
		let g = match q.3 {
			Some(g) => {
				let g = generate_pattern(g);
				quote!(Some(#g?))
			}
			None => quote!(None),
		};

		quote!(::treeldr::rdf_types::Quad(#s?, #p?, #o?, #g))
	});

	quote!((|| Some([#(#quads),*]))())
}

fn inputs_to_array(inputs: &[Pattern<Term>]) -> TokenStream {
	let items = inputs.iter().map(generate_pattern);
	quote!((|| Some([#(#items?),*]))())
}

fn generate_pattern(pattern: &Pattern<Term>) -> TokenStream {
	match pattern {
		Pattern::Var(i) => quote!(Some(::treeldr::Pattern::Var(#i))),
		Pattern::Resource(term) => match term {
			Term::Id(Id::Blank(_)) => panic!(),
			Term::Id(Id::Iri(iri)) => {
				let iri = iri.as_str();
				quote!(
					rdf.iri_interpretation(unsafe { ::treeldr::iref::Iri::new_unchecked(#iri) }).map(::treeldr::Pattern::Resource)
				)
			}
			Term::Literal(l) => {
				use rdf_types::literal;
				let value = l.value().as_str();
				let ty = match l.type_() {
					literal::Type::Any(iri) => {
						let iri = iri.as_str();
						quote!(::treeldr::rdf_types::literal::Type::Any(unsafe { ::treeldr::iref::Iri::new_unchecked(#iri) }))
					}
					literal::Type::LangString(_tag) => {
						todo!("lang string support")
					}
				};

				quote!(
					rdf.literal_interpretation(#value, #ty).map(::treeldr::Pattern::Resource)
				)
			}
		},
	}
}

fn term_to_datatype(term: &Term) -> Result<TokenStream, Error> {
	match term {
		Term::Id(Id::Iri(iri)) => {
			let iri = iri.as_str();
			Ok(quote!(unsafe { ::treeldr::iref::Iri::new_unchecked(#iri) }))
		}
		other => Err(Error::InvalidDatatype(other.to_string())),
	}
}

fn generate_data(
	ident: &syn::Ident,
	extra: &mut Option<TokenStream>,
	intro: u32,
	dataset: &Dataset,
	resource: &Pattern<Term>,
	datatype: &Term,
) -> Result<TokenStream, Error> {
	let dataset = dataset_to_array(dataset);
	let resource = generate_pattern(resource);
	let expected_ty_iri = term_to_datatype(datatype)?;

	*extra = Some(quote! {
		impl ::treeldr::de::FromRdfLiteral for #ident {
			fn from_rdf_literal(s: &str) -> Result<Self, ::treeldr::de::InvalidLiteral> {
				::treeldr::de::FromRdfLiteral::from_rdf_literal(s).map(Self)
			}
		}
	});

	Ok(quote! {
		let mut substitution = ::treeldr::pattern::Substitution::from_inputs(inputs);
		substitution.intro(#intro);

		let substitution = ::treeldr::de::Matching::new(
			dataset,
			substitution.clone(),
			::treeldr::utils::QuadsExt::with_default_graph(
				#dataset
					.ok_or(::treeldr::DeserializeError::MissingData)?
					.iter()
					.map(::treeldr::rdf_types::Quad::borrow_components),
				current_graph
			),
		)
		.into_required_unique()?;

		let resource = #resource.ok_or(::treeldr::DeserializeError::MissingData)?.apply(&substitution).into_resource().unwrap();

		let mut result = None;

		let expected_ty_iri = #expected_ty_iri;
		let mut has_literal = false;
		for l in rdf.interpretation.literals_of(&resource) {
			has_literal = true;
			let literal = rdf.vocabulary.literal(l).unwrap();
			let ty_iri = match literal.type_() {
				::treeldr::rdf_types::literal::Type::Any(i) => {
					rdf.vocabulary.iri(i).unwrap()
				},
				::treeldr::rdf_types::literal::Type::LangString(_) => {
					::treeldr::rdf_types::RDF_LANG_STRING
				}
			};

			if ty_iri == expected_ty_iri {
				if let Ok(value) = ::treeldr::de::FromRdfLiteral::from_rdf_literal(literal.value().as_str()) {
					if result.replace(value).is_some() {
						return Err(::treeldr::DeserializeError::AmbiguousLiteralValue)
					}
				}
			}
		}

		match result {
			Some(r) => Ok(r),
			None => if has_literal {
				Err(::treeldr::DeserializeError::LiteralTypeMismatch)
			} else {
				Err(::treeldr::DeserializeError::ExpectedLiteral)
			}
		}
	})
}
