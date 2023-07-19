use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

/// `Quads` trait implementation.
pub struct QuadsImpl {
	pub type_path: syn::Type,
	pub iterator_ty: IteratorType,
	pub bounds: Vec<syn::WherePredicate>,
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

			impl<V, I> ::treeldr_rust_prelude::rdf::QuadsAndValues<V, I> for #type_path
			where
				V: ::treeldr_rust_prelude::rdf_types::VocabularyMut,
				I: ::treeldr_rust_prelude::rdf_types::InterpretationMut + ::treeldr_rust_prelude::rdf_types::IriInterpretationMut<V::Iri> + ::treeldr_rust_prelude::rdf_types::LiteralInterpretationMut<V::Literal>,
				I::Resource: Clone,
				#(#bounds),*
			{
				type QuadsAndValues<'r> = #iterator_ident<'r, I::Resource> where Self: 'r, I::Resource: 'r;

				fn unbound_rdf_quads_and_values<'r>(
					&'r self,
					vocabulary: &mut V,
					interpretation: &mut I
				) -> (Option<I::Resource>, Self::QuadsAndValues<'r>)
				where
					I::Resource: 'r
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
	Enum(IteratorEnum),
}

impl IteratorType {
	pub fn ident(&self) -> &Ident {
		match self {
			Self::Struct(s) => &s.ident,
			Self::Enum(e) => &e.ident,
		}
	}
}

impl ToTokens for IteratorType {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self {
			Self::Struct(s) => s.to_tokens(tokens),
			Self::Enum(e) => e.to_tokens(tokens),
		}
	}
}

impl<'a> ToTokens for Init<'a, IteratorType> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self.0 {
			IteratorType::Struct(s) => Init(s).to_tokens(tokens),
			IteratorType::Enum(e) => Init(e).to_tokens(tokens),
		}
	}
}

impl<'a> ToTokens for IteratorImpl<'a, IteratorType> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self.0 {
			IteratorType::Struct(s) => IteratorImpl(s, self.1).to_tokens(tokens),
			IteratorType::Enum(e) => IteratorImpl(e, self.1).to_tokens(tokens),
		}
	}
}

pub struct IteratorStruct {
	pub ident: Ident,
	pub fields: Vec<IteratorField>,
	pub id_init: TokenStream,
	pub next_body: TokenStream,
}

impl ToTokens for IteratorStruct {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.ident;
		let fields = &self.fields;

		tokens.extend(quote! {
			pub struct #ident<'r, R> {
				id_: Option<R>,
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
			let id = #id_init;
			(
				Some(id.clone()),
				#ident {
					id_: Some(id),
					#(#fields_init),*
				}
			)
		})
	}
}

impl<'a> ToTokens for IteratorImpl<'a, IteratorStruct> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.0.ident;
		let bounds = self.1;
		let next_body = &self.0.next_body;

		tokens.extend(quote! {
			impl<'r, V, I> ::treeldr_rust_prelude::RdfIterator<V, I> for #ident<'r, I::Resource>
			where
				V: ::treeldr_rust_prelude::rdf_types::VocabularyMut,
				I: ::treeldr_rust_prelude::rdf_types::InterpretationMut + ::treeldr_rust_prelude::rdf_types::IriInterpretationMut<V::Iri> + ::treeldr_rust_prelude::rdf_types::LiteralInterpretationMut<V::Literal>,
				I::Resource: 'r + Clone,
				#(#bounds),*
			{
				type Item = ::treeldr_rust_prelude::rdf::QuadOrValue<I::Resource>;

				fn next_with(
					&mut self,
					vocabulary: &mut V,
					interpretation: &mut I,
					graph: Option<&I::Resource>
				) -> Option<Self::Item> {
					#next_body
				}
			}
		})
	}
}

pub struct IteratorEnum {
	pub ident: Ident,
	pub variants: Vec<IteratorVariant>,
}

impl ToTokens for IteratorEnum {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.ident;
		let variants = &self.variants;

		tokens.extend(quote! {
			pub enum #ident<'r, R> {
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
				quote!(Self::#v_ident(value) => {
					let (id, inner) = value.unbound_rdf_quads_and_values(vocabulary, interpretation);
					(id, #ident::#v_ident(inner))
				})
			} else {
				quote!(Self::#v_ident => (None, #ident::#v_ident))
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
				quote!(Self::#v_ident(inner) => inner.next_with(vocabulary, interpretation, graph))
			} else {
				quote!(Self::#v_ident => None)
			}
		});

		tokens.extend(quote! {
			impl<'r, V, I> ::treeldr_rust_prelude::RdfIterator<V, I> for #ident<'r, I::Resource>
			where
				V: ::treeldr_rust_prelude::rdf_types::VocabularyMut,
				I: ::treeldr_rust_prelude::rdf_types::InterpretationMut + ::treeldr_rust_prelude::rdf_types::IriInterpretationMut<V::Iri> + ::treeldr_rust_prelude::rdf_types::LiteralInterpretationMut<V::Literal>,
				I::Resource: 'r + Clone,
				#(#bounds),*
			{
				type Item = ::treeldr_rust_prelude::rdf::QuadOrValue<I::Resource>;

				fn next_with(
					&mut self,
					vocabulary: &mut V,
					interpretation: &mut I,
					graph: Option<&I::Resource>
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
	pub init: TokenStream,
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
	pub ty: Option<syn::Type>,
}

impl ToTokens for IteratorVariant {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let ty = &self.ty;
		tokens.extend(quote!(#ident(#ty)))
	}
}
