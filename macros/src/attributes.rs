use proc_macro2::{Span, TokenTree};
use syn::{punctuated::Punctuated, spanned::Spanned, Ident, Token, WherePredicate};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("expected an attribute of the form `#[serde_indexed(...)]`")]
	ExpectedMetaList(Span),

	#[error(transparent)]
	Parse(#[from] syn::parse::Error),
}

impl Error {
	pub fn span(&self) -> Span {
		match self {
			Self::ExpectedMetaList(s) => *s,
			Self::Parse(e) => e.span(),
		}
	}
}

#[derive(Debug, Default)]
pub struct TypeExprAttributes {
	pub with: Option<Ident>,
}

impl TypeExprAttributes {
	pub fn parse_attributes(attrs: &[syn::Attribute]) -> Result<Self, Error> {
		let mut result = Self::default();

		for attr in attrs {
			if attr.path().is_ident("serde_indexed") {
				match &attr.meta {
					syn::Meta::List(list) => {
						let a: Self = syn::parse2(list.tokens.clone())?;
						result.merge_with(a)
					}
					_ => return Err(Error::ExpectedMetaList(attr.span())),
				}
			}
		}

		Ok(result)
	}

	pub fn merge_with(&mut self, other: Self) {
		if let Some(w) = other.with {
			self.with = Some(w)
		}
	}
}

impl syn::parse::Parse for TypeExprAttributes {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let mut result = Self::default();

		let attributes: Punctuated<TypeExprAttribute, Token![,]> =
			Punctuated::parse_terminated(input)?;
		for attr in attributes {
			match attr {
				TypeExprAttribute::With(ident) => {
					result.with = Some(ident);
				}
			}
		}

		Ok(result)
	}
}

pub enum TypeExprAttribute {
	With(Ident),
}

impl syn::parse::Parse for TypeExprAttribute {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let id: syn::Ident = input.parse()?;
		if id == "with" {
			let _: Token![=] = input.parse()?;
			input.parse().map(Self::With)
		} else {
			Err(syn::Error::new(id.span(), "unexpected ident"))
		}
	}
}

#[derive(Default)]
pub struct Attributes {
	pub deserialize_bounds: Vec<WherePredicate>,
}

impl Attributes {
	pub fn parse_attributes(attrs: &[syn::Attribute]) -> Result<Self, Error> {
		let mut result = Self::default();

		for attr in attrs {
			if attr.path().is_ident("serde_indexed") {
				match &attr.meta {
					syn::Meta::List(list) => {
						let a: Self = syn::parse2(list.tokens.clone())?;
						result.merge_with(a)
					}
					_ => return Err(Error::ExpectedMetaList(attr.span())),
				}
			}
		}

		Ok(result)
	}

	pub fn merge_with(&mut self, other: Self) {
		self.deserialize_bounds.extend(other.deserialize_bounds);
	}
}

impl syn::parse::Parse for Attributes {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let mut result = Self::default();

		let attributes: Punctuated<Attribute, Token![,]> = Punctuated::parse_terminated(input)?;
		for attr in attributes {
			match attr {
				Attribute::DeserializeBounds(predicates) => {
					result.deserialize_bounds = predicates.into_iter().collect();
				}
			}
		}

		Ok(result)
	}
}

pub enum Attribute {
	DeserializeBounds(Punctuated<WherePredicate, Token![,]>),
}

impl syn::parse::Parse for Attribute {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let id: syn::Ident = input.parse()?;
		if id == "deserialize_bounds" {
			let content;
			let _ = syn::parenthesized!(content in input);
			let predicates: Punctuated<WherePredicate, Token![,]> =
				Punctuated::parse_terminated(&content)?;
			Ok(Self::DeserializeBounds(predicates))
		} else {
			Err(syn::Error::new(id.span(), "unexpected ident"))
		}
	}
}
