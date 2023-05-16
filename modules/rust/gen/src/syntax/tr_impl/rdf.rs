use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};

/// `Quads` trait implementation.
pub struct QuadsImpl {
	pub type_path: syn::Type,
	pub iterator_ty: IteratorType,
	pub bounds: Vec<syn::WherePredicate>
}

impl ToTokens for QuadsImpl {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let type_path = &self.type_path;
		let iterator_ty = &self.iterator_ty;
		let iterator_impl = IteratorImpl(&self.iterator_ty, &self.bounds);
		let iterator_ident = self.iterator_ty.ident();
		let bounds = &self.bounds;
		let iterator_init = Init(&self.iterator_ty);

		tokens.extend(quote! {
			#iterator_ty
			#iterator_impl

			impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::rdf::QuadsAndValues<N, V> for #type_path
			where
				N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
				N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
				#(#bounds),*
			{
				type QuadsAndValues<'a> = #iterator_ident<'a, N::Id, V> where Self: 'a, N::Id: 'a, V: 'a;

				fn unbound_rdf_quads_and_values<
					'a,
					G: ::treeldr_rust_prelude::rdf_types::Generator<N>
				>(
					&'a self,
					namespace: &mut N,
					generator: &mut G
				) -> Self::QuadsAndValues<'a>
				where
					N::Id: 'a,
					V: 'a
				{
					#iterator_init
				}
			}
		})
	}
}

struct Init<'a, T>(&'a T);

struct IteratorImpl<'a, T>(&'a T, &'a [syn::WherePredicate]);

pub enum IteratorType {
	Struct(IteratorStruct),
	Enum(IteratorEnum)
}

impl IteratorType {
	pub fn ident(&self) -> &Ident {
		match self {
			Self::Struct(s) => &s.ident,
			Self::Enum(e) => &e.ident
		}
	}
}

impl ToTokens for IteratorType {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self {
			Self::Struct(s) => s.to_tokens(tokens),
			Self::Enum(e) => e.to_tokens(tokens)
		}
	}
}

impl<'a> ToTokens for Init<'a, IteratorType> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self.0 {
			IteratorType::Struct(s) => Init(s).to_tokens(tokens),
			IteratorType::Enum(e) => Init(e).to_tokens(tokens)
		}
	}
}

impl<'a> ToTokens for IteratorImpl<'a, IteratorType> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self.0 {
			IteratorType::Struct(s) => IteratorImpl(s, self.1).to_tokens(tokens),
			IteratorType::Enum(e) => IteratorImpl(e, self.1).to_tokens(tokens)
		}
	}
}

pub struct IteratorStruct {
	pub ident: Ident,
	pub fields: Vec<IteratorField>,
	pub id_init: TokenStream,
	pub next_body: TokenStream
}

impl ToTokens for IteratorStruct {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.ident;
		let fields = &self.fields;

		tokens.extend(quote! {
			pub struct #ident<'a, I, V> {
				id_: Option<I>,
				#(#fields),*
			}
		})
	}
}

impl<'a> ToTokens for Init<'a, IteratorStruct> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.0.ident;
		let id_init = &self.0.id_init;
		let fields_init = self.0.fields.iter().map(|f| {
			let ident = &f.ident;
			let value = &f.init;
			quote!(#ident : #value)
		});

		tokens.extend(quote! {
			#ident {
				id_: Some(#id_init),
				#(#fields_init),*
			}
		})
	}
}

impl<'a> ToTokens for IteratorImpl<'a, IteratorStruct> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.0.ident;
		let bounds = self.1;
		let next_body = &self.0.next_body;

		tokens.extend(quote! {
			impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a> ::treeldr_rust_prelude::RdfIterator<N> for #ident<'a, N::Id, V>
			where
				N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
				N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
				#(#bounds),*
			{
				type Item = ::treeldr_rust_prelude::rdf::QuadOrValue<N::Id, V>;

				fn next_with<
					G: ::treeldr_rust_prelude::rdf_types::Generator<N>
				>(
					&mut self,
					vocabulary: &mut N,
					generator: &mut G,
					graph: Option<&N::Id>
				) -> Option<Self::Item> {
					#next_body
				}
			}
		})
	}
}

pub struct IteratorEnum {
	pub ident: Ident,
	pub variants: Vec<IteratorVariant>
}

impl ToTokens for IteratorEnum {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.ident;
		let variants = &self.variants;

		tokens.extend(quote! {
			pub enum #ident<'a, I, V> {
				#(#variants),*
			}
		})
	}
}

impl<'a> ToTokens for Init<'a, IteratorEnum> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.0.ident;
		let variants_init = self.0.variants.iter().map(|v| {
			let v_ident = &v.ident;
			if v.ty.is_some() {
				quote!(Self::#v_ident(value) => #ident::#v_ident(value.unbound_rdf_quads_and_values(namespace, generator)))
			} else {
				quote!(Self::#v_ident => #ident::#v_ident)
			}
		});

		tokens.extend(quote! {
			match self {
				#(#variants_init),*
			}
		})
	}
}

impl<'a> ToTokens for IteratorImpl<'a, IteratorEnum> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.0.ident;
		let bounds = self.1;
		let variants_next = self.0.variants.iter().map(|v| {
			let v_ident = &v.ident;
			if v.ty.is_some() {
				quote!(Self::#v_ident(inner) => inner.next_with(namespace, generator, graph))
			} else {
				quote!(Self::#v_ident => None)
			}
		});

		tokens.extend(quote! {
			impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a> ::treeldr_rust_prelude::RdfIterator<N> for #ident<'a, N::Id, V>
			where
				N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
				N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
				#(#bounds),*
			{
				type Item = ::treeldr_rust_prelude::rdf::QuadOrValue<N::Id, V>;

				fn next_with<
					G: ::treeldr_rust_prelude::rdf_types::Generator<N>
				>(
					&mut self,
					vocabulary: &mut N,
					generator: &mut G,
					graph: Option<&N::Id>
				) -> Option<Self::Item> {
					match self {
						#(#variants_next),*
					}
				}
			}
		})
	}
}

pub struct IteratorField {
	pub ident: Ident,
	pub ty: syn::Type,
	pub init: TokenStream
}

impl ToTokens for IteratorField {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let ty = &self.ty;
		tokens.extend(quote!(#ident : #ty))
	}
}

pub struct IteratorVariant {
	pub ident: Ident,
	pub ty: Option<syn::Type>
}

impl ToTokens for IteratorVariant {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let ty = &self.ty;
		tokens.extend(quote!(#ident(#ty)))
	}
}