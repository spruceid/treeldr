use locspan::Meta;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use treeldr::TId;

use crate::{syntax, GenerateSyntax};

pub struct Restricted {
	ident: Ident,
	base: TId<treeldr::Layout>,
	restrictions: Vec<Restriction>,
}

impl Restricted {
	pub fn new(ident: Ident, base: TId<treeldr::Layout>, restrictions: Vec<Restriction>) -> Self {
		Self {
			ident,
			base,
			restrictions,
		}
	}

	pub fn ident(&self) -> &Ident {
		&self.ident
	}

	pub fn base(&self) -> TId<treeldr::Layout> {
		self.base
	}
}

impl<M> GenerateSyntax<M> for Restricted {
	type Output = syntax::ty::primitive::Restricted;

	fn generate_syntax<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, crate::Error> {
		let mut restrictions = Vec::with_capacity(self.restrictions.len());
		for r in &self.restrictions {
			restrictions.push(r.generate_syntax(context, scope)?)
		}

		Ok(syntax::ty::primitive::Restricted {
			ident: self.ident.clone(),
			base: self.base.generate_syntax(context, scope)?,
			restrictions,
		})
	}
}

pub enum Restriction {
	MinInclusive(syn::Type, TokenStream),
	MinExclusive(syn::Type, TokenStream),
	MaxInclusive(syn::Type, TokenStream),
	MaxExclusive(syn::Type, TokenStream),
	MinLength(syn::Type, TokenStream),
	MaxLength(syn::Type, TokenStream),
}

impl Restriction {
	pub fn new<M>(Meta(r, _): Meta<treeldr::layout::primitive::RestrictionRef, M>) -> Self {
		use treeldr::layout::primitive::RestrictionRef;

		macro_rules! integer_lexical {
			( $r:ident : $ty:ty ) => {
				{
					use treeldr::layout::primitive::restriction::template::integer::RestrictionRef;
					let ty = syn::parse2(quote!($ty)).unwrap();
					match $r {
						RestrictionRef::MinInclusive(min) => {
							let bytes = min.to_signed_bytes_be();
							Self::MinInclusive(ty, quote!(unsafe { $ty::from_signed_bytes_be_unchecked(&[#(#bytes),*]) }))
						}
						RestrictionRef::MaxInclusive(max) => {
							let bytes = max.to_signed_bytes_be();
							Self::MaxInclusive(ty, quote!(unsafe { $ty::from_signed_bytes_be_unchecked(&[#(#bytes),*]) }))
						}
					}
				}
			};
		}

		macro_rules! integer_non_lexical {
			( $r:ident : $ty:ty ) => {
				{
					use treeldr::layout::primitive::restriction::template::integer::RestrictionRef;
					let ty = syn::parse2(quote!($ty)).unwrap();
					match $r {
						RestrictionRef::MinInclusive(min) => {
							Self::MinInclusive(ty, quote!(#min))
						}
						RestrictionRef::MaxInclusive(max) => {
							Self::MaxInclusive(ty, quote!(#max))
						}
					}
				}
			};
		}

		macro_rules! float {
			( $r:ident : $ty:ty, $fty:ty ) => {
				{
					use treeldr::layout::primitive::restriction::template::float::{RestrictionRef, Min, Max};
					let ty = syn::parse2(quote!($fty)).unwrap();
					match $r {
						RestrictionRef::Min(Min::Included(min)) => {
							let min: $fty = (*min).into();
							Self::MinInclusive(ty, quote!(#min))
						}
						RestrictionRef::Min(Min::Excluded(min)) => {
							let min: $fty = (*min).into();
							Self::MinExclusive(ty, quote!(#min))
						}
						RestrictionRef::Max(Max::Included(max)) => {
							let max: $fty = (*max).into();
							Self::MaxInclusive(ty, quote!(#max))
						}
						RestrictionRef::Max(Max::Excluded(max)) => {
							let max: $fty = (*max).into();
							Self::MaxExclusive(ty, quote!(#max))
						}
					}
				}
			};
		}

		macro_rules! string {
			( $r:ident ) => {
				{
					use treeldr::layout::primitive::restriction::template::string::RestrictionRef;
					match $r {
						RestrictionRef::MinLength(min) => {
							let lexical = min.to_string();
							let ty = syn::parse2(quote!(treeldr_rust_prelude::ty::NonNegativeInteger)).unwrap();
							Self::MinLength(ty, quote!(unsafe { $ty::new_unchecked(#lexical) }))
						},
						RestrictionRef::MaxLength(max) => {
							let lexical = max.to_string();
							let ty = syn::parse2(quote!(treeldr_rust_prelude::ty::NonNegativeInteger)).unwrap();
							Self::MaxLength(ty, quote!(unsafe { $ty::new_unchecked(#lexical) }))
						},
						RestrictionRef::Pattern(_) => {
							unimplemented!("string pattern constraint")
						}
					}
				}
			};
		}

		match r {
			RestrictionRef::Integer(r) => {
				use treeldr::layout::primitive::restriction::template::integer::RestrictionRef;
				let ty = syn::parse2(quote!(treeldr_rust_prelude::ty::Integer)).unwrap();
				match r {
					RestrictionRef::MinInclusive(min) => {
						let bytes = min.to_signed_bytes_be();
						Self::MinInclusive(
							ty,
							quote!(treeldr_rust_prelude::ty::Integer::from_signed_bytes_be(&[#(#bytes),*])),
						)
					}
					RestrictionRef::MaxInclusive(max) => {
						let bytes = max.to_signed_bytes_be();
						Self::MaxInclusive(
							ty,
							quote!(treeldr_rust_prelude::ty::Integer::from_signed_bytes_be(&[#(#bytes),*])),
						)
					}
				}
			}
			RestrictionRef::NonPositiveInteger(r) => {
				integer_lexical!(r: treeldr_rust_prelude::ty::NonPositiveInteger)
			}
			RestrictionRef::NonNegativeInteger(r) => {
				integer_lexical!(r: treeldr_rust_prelude::ty::NonNegativeInteger)
			}
			RestrictionRef::PositiveInteger(r) => {
				integer_lexical!(r: treeldr_rust_prelude::ty::PositiveInteger)
			}
			RestrictionRef::NegativeInteger(r) => {
				integer_lexical!(r: treeldr_rust_prelude::ty::NegativeInteger)
			}
			RestrictionRef::I64(r) => integer_non_lexical!(r: treeldr_rust_prelude::ty::I64),
			RestrictionRef::I32(r) => integer_non_lexical!(r: treeldr_rust_prelude::ty::I32),
			RestrictionRef::I16(r) => integer_non_lexical!(r: treeldr_rust_prelude::ty::I16),
			RestrictionRef::I8(r) => integer_non_lexical!(r: treeldr_rust_prelude::ty::I8),
			RestrictionRef::U64(r) => integer_non_lexical!(r: treeldr_rust_prelude::ty::U64),
			RestrictionRef::U32(r) => integer_non_lexical!(r: treeldr_rust_prelude::ty::U32),
			RestrictionRef::U16(r) => integer_non_lexical!(r: treeldr_rust_prelude::ty::U16),
			RestrictionRef::U8(r) => integer_non_lexical!(r: treeldr_rust_prelude::ty::U8),
			RestrictionRef::Float(r) => float!(r: treeldr_rust_prelude::ty::Float, f32),
			RestrictionRef::Double(r) => float!(r: treeldr_rust_prelude::ty::Double, f64),
			RestrictionRef::Base64Bytes(r) => string!(r),
			RestrictionRef::HexBytes(r) => string!(r),
			RestrictionRef::String(r) => string!(r),
			RestrictionRef::Boolean(_)
			| RestrictionRef::Time(_)
			| RestrictionRef::Date(_)
			| RestrictionRef::DateTime(_)
			| RestrictionRef::Iri(_)
			| RestrictionRef::Uri(_)
			| RestrictionRef::Url(_)
			| RestrictionRef::Bytes(_)
			| RestrictionRef::Cid(_) => unreachable!(),
		}
	}
}

impl<M> GenerateSyntax<M> for Restriction {
	type Output = syn::Expr;

	fn generate_syntax<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		_context: &crate::Context<V, M>,
		_scope: &crate::Scope,
	) -> Result<Self::Output, crate::Error> {
		match self {
			Self::MinInclusive(ty, value) => Ok(syn::parse2(
				quote!(treeldr_rust_prelude::restriction::MinInclusive::<#ty>::check(value, &#value)),
			)
			.unwrap()),
			Self::MinExclusive(ty, value) => Ok(syn::parse2(
				quote!(treeldr_rust_prelude::restriction::MinExclusive::<#ty>::check(value, &#value)),
			)
			.unwrap()),
			Self::MaxInclusive(ty, value) => Ok(syn::parse2(
				quote!(treeldr_rust_prelude::restriction::MaxInclusive::<#ty>::check(value, &#value)),
			)
			.unwrap()),
			Self::MaxExclusive(ty, value) => Ok(syn::parse2(
				quote!(treeldr_rust_prelude::restriction::MaxExclusive::<#ty>::check(value, &#value)),
			)
			.unwrap()),
			Self::MinLength(ty, value) => Ok(syn::parse2(
				quote!(treeldr_rust_prelude::restriction::MinLength::<#ty>::check(value, &#value)),
			)
			.unwrap()),
			Self::MaxLength(ty, value) => Ok(syn::parse2(
				quote!(treeldr_rust_prelude::restriction::MaxLength::<#ty>::check(value, &#value)),
			)
			.unwrap()),
		}
	}
}
