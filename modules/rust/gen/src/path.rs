use proc_macro2::{Ident, TokenStream};
use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, IriIndex};

use crate::{ty, Context, Error, GenerateSyntax, Scope};

#[derive(Default, Clone)]
pub struct Path {
	segments: Vec<Segment>,

	// Required parameters.
	params: ty::Parameters,
}

impl Path {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn len(&self) -> usize {
		self.segments.len()
	}

	pub fn is_empty(&self) -> bool {
		self.segments.is_empty()
	}

	pub fn segments(&self) -> std::slice::Iter<Segment> {
		self.segments.iter()
	}

	pub fn longest_common_prefix(&self, other: &Self) -> Self {
		self.segments()
			.zip(other.segments())
			.take_while(|(a, b)| a == b)
			.map(|(s, _)| s)
			.cloned()
			.collect()
	}

	pub fn to(&self, other: &Self) -> Self {
		let lcp = self.longest_common_prefix(other);
		let mut path = Path::new();

		for _ in lcp.len()..self.len() {
			path.push(Segment::Super)
		}

		for i in lcp.len()..other.len() {
			path.push(other.segments[i].clone())
		}

		path.params = other.params;
		path
	}

	pub fn push(&mut self, segment: impl Into<Segment>) {
		self.segments.push(segment.into())
	}

	pub fn parameters(&self) -> &ty::Parameters {
		&self.params
	}

	pub fn parameters_mut(&mut self) -> &mut ty::Parameters {
		&mut self.params
	}
}

impl FromIterator<Segment> for Path {
	fn from_iter<I: IntoIterator<Item = Segment>>(iter: I) -> Self {
		Self {
			segments: iter.into_iter().collect(),
			params: ty::Parameters::default(),
		}
	}
}

impl<M> GenerateSyntax<M> for Path {
	type Output = syn::Path;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		_context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		let mut segments = syn::punctuated::Punctuated::new();

		for s in self.segments() {
			let ident = match s {
				Segment::Super => Ident::new("super", proc_macro2::Span::call_site()),
				Segment::Ident(id) => id.clone(),
			};

			segments.push(syn::PathSegment {
				ident,
				arguments: syn::PathArguments::None,
			});
		}

		if !self.params.is_empty() {
			let mut args = syn::punctuated::Punctuated::new();

			for p in self.params.iter() {
				args.push(scope.bound_params().get(p).unwrap().into_owned())
			}

			segments.last_mut().unwrap().arguments =
				syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
					colon2_token: Some(syn::token::PathSep::default()),
					lt_token: Default::default(),
					args,
					gt_token: Default::default(),
				})
		}

		Ok(syn::Path {
			leading_colon: None,
			segments,
		})
	}
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Segment {
	Super,
	Ident(proc_macro2::Ident),
}

impl From<proc_macro2::Ident> for Segment {
	fn from(value: proc_macro2::Ident) -> Self {
		Self::Ident(value)
	}
}

impl quote::ToTokens for Segment {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self {
			Self::Super => tokens.extend(quote! { super }),
			Self::Ident(id) => id.to_tokens(tokens),
		}
	}
}
