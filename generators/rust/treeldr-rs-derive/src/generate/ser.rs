use proc_macro2::{Span, TokenStream};
use quote::quote;
use rdf_types::{Id, Term};
use syn::DeriveInput;
use treeldr_layouts::{
	layout::{DataLayout, LiteralLayout},
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

	let body = match layout {
		Layout::Always => {
			unreachable!()
		}
		Layout::Literal(layout) => match layout {
			LiteralLayout::Id(layout) => {
				let intro = layout.intro;
				let dataset = dataset_to_array(&layout.dataset);

				quote! {
					let env = env.intro(rdf, #intro);
					env.instantiate_dataset(&#dataset, output);
					let value: &str = AsRef::<str>::as_ref(&self.0);

					match ::treeldr::rdf_types::BlankId::new(value) {
						Ok(value) => {
							let id = rdf.vocabulary.insert_blank_id(value);
							rdf.interpretation.assign_blank_id(inputs[0].clone(), id);
							Ok(())
						}
						Err(_) => match ::treeldr::iref::Iri::new(value) {
							Ok(value) => {
								let id = rdf.vocabulary.insert(value);
								rdf.interpretation.assign_iri(inputs[0].clone(), id);
								Ok(())
							}
							Err(_) => {
								Err(::treeldr::SerializeError::InvalidId(value.to_owned()))
							}
						}
					}
				}
			}
			LiteralLayout::Data(layout) => match layout {
				DataLayout::Unit(layout) => {
					let intro = layout.intro;
					let dataset = dataset_to_array(&layout.dataset);

					quote! {
						let env = env.intro(rdf, #intro);
						env.instantiate_dataset(&#dataset, output);
						Ok(())
					}
				}
				DataLayout::Boolean(layout) => {
					let intro = layout.intro;
					let dataset = dataset_to_array(&layout.dataset);
					let datatype = term_to_datatype_owned(&layout.datatype)?;
					let target = pattern_interpretation(&layout.resource);

					quote! {
						let env = env.intro(rdf, #intro);
						env.instantiate_dataset(&#dataset, output);

						let literal = rdf.vocabulary_literal_owned(::treeldr::rdf_types::Literal::new(
							self.0.to_string(),
							#datatype
						));
						rdf.interpretation.assign_literal(#target, literal);
						Ok(())
					}
				}
				DataLayout::Number(layout) => {
					let intro = layout.intro;
					let dataset = dataset_to_array(&layout.dataset);
					let datatype = term_to_datatype_owned(&layout.datatype)?;
					let target = pattern_interpretation(&layout.resource);

					quote! {
						let env = env.intro(rdf, #intro);
						env.instantiate_dataset(&#dataset, output);

						let literal = rdf.vocabulary_literal_owned(::treeldr::rdf_types::Literal::new(
							self.0.to_string(),
							#datatype
						));
						rdf.interpretation.assign_literal(#target, literal);
						Ok(())
					}
				}
				DataLayout::ByteString(layout) => {
					let intro = layout.intro;
					let dataset = dataset_to_array(&layout.dataset);
					let datatype = term_to_datatype_owned(&layout.datatype)?;
					let target = pattern_interpretation(&layout.resource);

					quote! {
						let env = env.intro(rdf, #intro);
						env.instantiate_dataset(&#dataset, output);

						let literal = rdf.vocabulary_literal_owned(::treeldr::rdf_types::Literal::new(
							self.0.to_string(),
							#datatype
						));
						rdf.interpretation.assign_literal(#target, literal);
						Ok(())
					}
				}
				DataLayout::TextString(layout) => {
					let intro = layout.intro;
					let dataset = dataset_to_array(&layout.dataset);
					let datatype = term_to_datatype(&layout.datatype)?;
					let target = pattern_interpretation(&layout.resource);

					quote! {
						let env = env.intro(rdf, #intro);
						env.instantiate_dataset(&#dataset, output);

						let literal = rdf.vocabulary_literal(::treeldr::rdf_types::Literal::new(
							::std::convert::AsRef::<str>::as_ref(&self.0),
							#datatype
						));
						rdf.interpretation.assign_literal(#target, literal);
						Ok(())
					}
				}
			},
		},
		Layout::Product(layout) => {
			let intro = layout.intro;
			let dataset = dataset_to_array(&layout.dataset);

			let fields = layout.fields.iter().map(|(name, field)| {
				let field_ident = syn::Ident::new(name, Span::call_site());
				let field_intro = field.intro;
				let field_dataset = dataset_to_array(&field.dataset);
				let field_layout = input
					.type_map
					.get(field.value.layout.id().as_iri().unwrap())
					.unwrap();
				let field_inputs = inputs_to_array(&field.value.input);
				let field_graph = match &field.value.graph {
					Some(None) => quote!(None),
					Some(Some(g)) => {
						let g = generate_pattern(g);
						quote!(Some(env.instantiate_pattern(#g)))
					}
					None => quote!(current_graph.cloned()),
				};

				let m = field.value.input.len();

				quote! {
					{
						let env = env.intro(rdf, #field_intro);
						env.instantiate_dataset(&#field_dataset, output);
						<#field_layout as ::treeldr::SerializeLd<#m, V, I>>::serialize_ld_with(
							&self.#field_ident,
							rdf,
							&env.instantiate_patterns(&#field_inputs),
							#field_graph.as_ref(),
							output
						)?;
					}
				}
			});

			quote! {
				let env = env.intro(rdf, #intro);
				env.instantiate_dataset(&#dataset, output);
				#(#fields)*
				Ok(())
			}
		}
		Layout::Sum(layout) => {
			let intro = layout.intro;
			let dataset = dataset_to_array(&layout.dataset);

			let variants = layout.variants.iter().map(|variant| {
				let variant_ident = syn::Ident::new(&variant.name, Span::call_site());
				let variant_intro = variant.intro;
				let variant_dataset = dataset_to_array(&variant.dataset);
				let variant_layout = input
					.type_map
					.get(variant.value.layout.id().as_iri().unwrap())
					.unwrap();
				let variant_inputs = inputs_to_array(&variant.value.input);
				let variant_graph = match &variant.value.graph {
					Some(None) => quote!(None),
					Some(Some(g)) => {
						let g = generate_pattern(g);
						quote!(Some(env.instantiate_pattern(#g)))
					}
					None => quote!(current_graph.cloned()),
				};

				let m = variant.value.input.len();

				quote! {
					Self::#variant_ident(value) => {
						let env = env.intro(rdf, #variant_intro);
						env.instantiate_dataset(&#variant_dataset, output);
						<#variant_layout as ::treeldr::SerializeLd<#m, V, I>>::serialize_ld_with(
							value,
							rdf,
							&env.instantiate_patterns(&#variant_inputs),
							#variant_graph.as_ref(),
							output
						)
					}
				}
			});

			quote! {
				let env = env.intro(rdf, #intro);
				env.instantiate_dataset(&#dataset, output);
				match self {
					#(#variants)*
				}
			}
		}
		Layout::List(_) => {
			todo!("list")
		}
		Layout::Never => {
			unreachable!()
		}
	};

	Ok(quote! {
		impl<V, I> ::treeldr::SerializeLd<#n, V, I> for #ident
		where
			V: ::treeldr::rdf_types::VocabularyMut<Value = String, Type = ::treeldr::RdfType<V>>,
			I: ::treeldr::rdf_types::InterpretationMut<V> + ::treeldr::rdf_types::TermInterpretationMut<V::Iri, V::BlankId, V::Literal> + ::treeldr::rdf_types::ReverseTermInterpretationMut<Iri = V::Iri, BlankId = V::BlankId, Literal = V::Literal>,
			I::Resource: Clone + Ord
		{
			fn serialize_ld_with(
				&self,
				rdf: &mut ::treeldr::RdfContextMut<V, I>,
				inputs: &[I::Resource; #n],
				current_graph: Option<&I::Resource>,
				output: &mut ::treeldr::grdf::BTreeDataset<I::Resource>
			) -> Result<(), ::treeldr::SerializeError> {
				let env = ::treeldr::Environment::Root(inputs);
				#body
			}
		}
	})
}

fn dataset_to_array(dataset: &Dataset) -> TokenStream {
	let quads = dataset.quads().map(|q| {
		let s = generate_pattern(q.0);
		let p = generate_pattern(q.1);
		let o = generate_pattern(q.2);
		let g = match q.3 {
			Some(g) => {
				let g = generate_pattern(g);
				quote!(Some(#g))
			}
			None => quote!(None),
		};

		quote!(::treeldr::rdf_types::Quad(#s, #p, #o, #g))
	});

	quote!([#(#quads),*])
}

fn inputs_to_array(inputs: &[Pattern<Term>]) -> TokenStream {
	let items = inputs.iter().map(|p| generate_pattern(p));
	quote!([#(#items),*])
}

fn generate_pattern(pattern: &Pattern<Term>) -> TokenStream {
	match pattern {
		Pattern::Var(i) => quote!(::treeldr::Pattern::Var(#i)),
		Pattern::Resource(term) => match term {
			Term::Id(Id::Blank(_)) => panic!(),
			Term::Id(Id::Iri(iri)) => {
				let iri = iri.as_str();
				quote!(::treeldr::Pattern::Resource(
					rdf.interpret_iri(unsafe { ::treeldr::iref::Iri::new_unchecked(#iri) })
				))
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

				quote!(::treeldr::Pattern::Resource(
					rdf.interpret_literal(#value, #ty)
				))
			}
		},
	}
}

fn term_interpretation(term: &Term) -> TokenStream {
	match term {
		Term::Id(Id::Iri(iri)) => {
			let iri = iri.as_str();
			quote!(rdf.interpret_iri(unsafe { ::treeldr::iref::Iri::new_unchecked(#iri) }))
		}
		Term::Id(Id::Blank(blank_id)) => {
			let blank_id = blank_id.as_str();
			quote!(rdf.interpret_blank_id(unsafe { ::treeldr::rdf_types::BlankId::new_unchecked(#blank_id) }))
		}
		Term::Literal(literal) => {
			use rdf_types::literal;
			let value = literal.value();
			let ty = match literal.type_() {
				literal::Type::Any(iri) => {
					let iri = iri.as_str();
					quote!(::treeldr::rdf_types::literal::Type::Any(unsafe { ::treeldr::iref::Iri::new_unchecked(#iri) }))
				}
				literal::Type::LangString(_tag) => {
					todo!("lang string support")
				}
			};

			quote!(rdf.interpret_literal(#value, #ty))
		}
	}
}

fn pattern_interpretation(pattern: &Pattern<Term>) -> TokenStream {
	match pattern {
		Pattern::Var(i) => {
			let i = *i as usize;
			quote!(inputs[#i].clone())
		}
		Pattern::Resource(term) => term_interpretation(term),
	}
}

fn term_to_datatype(term: &Term) -> Result<TokenStream, Error> {
	match term {
		Term::Id(Id::Iri(iri)) => {
			let iri = iri.as_str();
			Ok(
				quote!(::treeldr::rdf_types::literal::Type::Any(unsafe { ::treeldr::iref::Iri::new_unchecked(#iri) })),
			)
		}
		other => Err(Error::InvalidDatatype(other.to_string())),
	}
}

fn term_to_datatype_owned(term: &Term) -> Result<TokenStream, Error> {
	match term {
		Term::Id(Id::Iri(iri)) => {
			let iri = iri.as_str();
			Ok(
				quote!(::treeldr::rdf_types::literal::Type::Any(unsafe { ::treeldr::iref::Iri::new_unchecked(#iri) }.to_owned())),
			)
		}
		other => Err(Error::InvalidDatatype(other.to_string())),
	}
}
