use proc_macro2::{TokenStream, Span};
use quote::quote;
use rdf_types::{Term, Id};
use syn::DeriveInput;
use treeldr_layouts::{Layout, Dataset, Pattern};

use crate::parse::parse;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	Parse(#[from] crate::parse::Error),

	#[error(transparent)]
	Build(#[from] treeldr_layouts::abs::syntax::Error)
}

impl Error {
	pub fn span(&self) -> Span {
		match self {
			Self::Parse(e) => e.span(),
			Self::Build(_) => Span::call_site()
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
		},
		Layout::Literal(_) => {
			todo!()
		}
		Layout::Product(layout) => {
			let intro = layout.intro;
			let dataset = dataset_to_array(&layout.dataset);

			let fields = layout.fields.iter().map(|(name, field)| {
				let field_ident = syn::Ident::new(name, Span::call_site());
				let field_intro = field.intro;
				let field_dataset = dataset_to_array(&field.dataset);
				let field_layout = input.type_map.get(field.value.layout.id().as_iri().unwrap()).unwrap();
				let field_inputs = inputs_to_array(&field.value.input);
				let field_graph = match &field.value.graph {
					Some(None) => quote!(None),
					Some(Some(g)) => {
						let g = generate_pattern(g);
						quote!(Some(env.instantiate_pattern(#g)?))
					},
					None => quote!(current_graph.cloned()),
				};

				let m = field.value.input.len();

				quote! {
					let env = env.intro(rdf, #field_intro);
					env.instantiate_dataset(#field_dataset, output)?;
					<#field_layout as ::treeldr::SerializeLd<#m, V, I>>::serialize_ld_with(
						&self.#field_ident,
						rdf,
						env.instantiate_patterns(#field_inputs)?,
						#field_graph,
						output
					)?;
				}
			});

			quote! {
				let env = env.intro(rdf, #intro);
				env.instantiate_dataset(#dataset, output)?;
				#(#fields)*
				Ok(())
			}
		}
		Layout::Sum(_) => {
			todo!()
		}
		Layout::List(_) => {
			todo!()
		}
		Layout::Never => {
			unreachable!()
		}
	};

	Ok(quote! {
		impl<V, I> ::treeldr::SerializeLd<#n, V, I> for #ident {
			fn serialize_ld_with(
				&self,
				rdf: ::treeldr::RdfContext<V, I>,
				inputs: [I::Resource; #n],
				current_graph: Option<&I::Resource>,
				output: &mut ::treeldr::grdf::BTreeDataset<R>
			) -> Result<(), SerializeError> {
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
			None => quote!(None)
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
		Pattern::Resource(term) => {
			match term {
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
							todo!()
						}
					};

					quote!(::treeldr::Pattern::Resource(
						rdf.interpret_literal(#value, #ty)
					))
				}
			}
		}
	}
}