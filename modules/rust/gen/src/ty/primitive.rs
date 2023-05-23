use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{layout::Primitive, BlankIdIndex, IriIndex};

use crate::{Context, Error, GenerateSyntax, Referenced, Scope};

mod restricted;

pub use restricted::{Restricted, Restriction};

impl<M> GenerateSyntax<M> for treeldr::layout::Primitive {
	type Output = syn::Type;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		_context: &Context<V, M>,
		_scope: &Scope,
	) -> Result<Self::Output, Error> {
		match self {
			Self::Boolean => Ok(syn::parse2(quote! { bool }).unwrap()),
			Self::Integer => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::ty::Integer }).unwrap())
			}
			Self::NonNegativeInteger => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::ty::NonNegativeInteger }).unwrap())
			}
			Self::NonPositiveInteger => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::ty::NonPositiveInteger }).unwrap())
			}
			Self::PositiveInteger => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::ty::PositiveInteger }).unwrap())
			}
			Self::NegativeInteger => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::ty::NegativeInteger }).unwrap())
			}
			Self::I64 => Ok(syn::parse2(quote! { i64 }).unwrap()),
			Self::I32 => Ok(syn::parse2(quote! { i32 }).unwrap()),
			Self::I16 => Ok(syn::parse2(quote! { i16 }).unwrap()),
			Self::I8 => Ok(syn::parse2(quote! { i8 }).unwrap()),
			Self::U64 => Ok(syn::parse2(quote! { u64 }).unwrap()),
			Self::U32 => Ok(syn::parse2(quote! { u32 }).unwrap()),
			Self::U16 => Ok(syn::parse2(quote! { u16 }).unwrap()),
			Self::U8 => Ok(syn::parse2(quote! { u8 }).unwrap()),
			Self::Float => Ok(syn::parse2(quote! { f32 }).unwrap()),
			Self::Double => Ok(syn::parse2(quote! { f64 }).unwrap()),
			Self::Base64Bytes => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::ty::Base64BytesBuf }).unwrap())
			}
			Self::HexBytes => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::ty::HexBytesBuf }).unwrap())
			}
			Self::Bytes => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::ty::BytesBuf }).unwrap())
			}
			Self::String => Ok(syn::parse2(quote! { ::std::string::String }).unwrap()),
			Self::Date => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::chrono::NaiveDate }).unwrap())
			}
			Self::DateTime => Ok(syn::parse2(
				quote! { ::treeldr_rust_prelude::chrono::DateTime<::treeldr_rust_prelude::chrono::Utc> },
			)
			.unwrap()),
			Self::Time => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::chrono::NaiveTime }).unwrap())
			}
			Self::Url => Ok(syn::parse2(quote! { ::treeldr_rust_prelude::iref::IriBuf }).unwrap()),
			Self::Uri => Ok(syn::parse2(quote! { ::treeldr_rust_prelude::iref::IriBuf }).unwrap()),
			Self::Iri => Ok(syn::parse2(quote! { ::treeldr_rust_prelude::iref::IriBuf }).unwrap()),
			Self::Cid => Ok(syn::parse2(quote! { ::treeldr_rust_prelude::ty::CidBuf }).unwrap()),
		}
	}
}

impl<M> GenerateSyntax<M> for Referenced<treeldr::layout::Primitive> {
	type Output = syn::Type;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		_context: &Context<V, M>,
		_scope: &Scope,
	) -> Result<Self::Output, Error> {
		match self.0 {
			Primitive::Boolean => Ok(syn::parse2(quote! { bool }).unwrap()),
			Primitive::Integer => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::ty::Integer }).unwrap())
			}
			Primitive::NonNegativeInteger => Ok(syn::parse2(
				quote! { &::treeldr_rust_prelude::ty::NonNegativeInteger },
			)
			.unwrap()),
			Primitive::NonPositiveInteger => Ok(syn::parse2(
				quote! { &::treeldr_rust_prelude::ty::NonPositiveInteger },
			)
			.unwrap()),
			Primitive::PositiveInteger => {
				Ok(syn::parse2(quote! { &::treeldr_rust_prelude::ty::PositiveInteger }).unwrap())
			}
			Primitive::NegativeInteger => {
				Ok(syn::parse2(quote! { &::treeldr_rust_prelude::ty::NegativeInteger }).unwrap())
			}
			Primitive::I64 => Ok(syn::parse2(quote! { i64 }).unwrap()),
			Primitive::I32 => Ok(syn::parse2(quote! { i32 }).unwrap()),
			Primitive::I16 => Ok(syn::parse2(quote! { i16 }).unwrap()),
			Primitive::I8 => Ok(syn::parse2(quote! { i8 }).unwrap()),
			Primitive::U64 => Ok(syn::parse2(quote! { u64 }).unwrap()),
			Primitive::U32 => Ok(syn::parse2(quote! { u32 }).unwrap()),
			Primitive::U16 => Ok(syn::parse2(quote! { u16 }).unwrap()),
			Primitive::U8 => Ok(syn::parse2(quote! { u8 }).unwrap()),
			Primitive::Float => Ok(syn::parse2(quote! { f32 }).unwrap()),
			Primitive::Double => Ok(syn::parse2(quote! { f64 }).unwrap()),
			Primitive::Base64Bytes => {
				Ok(syn::parse2(quote! { &::treeldr_rust_prelude::ty::Base64Bytes }).unwrap())
			}
			Primitive::HexBytes => {
				Ok(syn::parse2(quote! { &::treeldr_rust_prelude::ty::HexBytes }).unwrap())
			}
			Primitive::Bytes => {
				Ok(syn::parse2(quote! { &::treeldr_rust_prelude::ty::Bytes }).unwrap())
			}
			Primitive::String => Ok(syn::parse2(quote! { &str }).unwrap()),
			Primitive::Date => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::chrono::NaiveDate }).unwrap())
			}
			Primitive::DateTime => Ok(syn::parse2(
				quote! { ::treeldr_rust_prelude::chrono::DateTime<::treeldr_rust_prelude::chrono::Utc> },
			)
			.unwrap()),
			Primitive::Time => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::chrono::NaiveTime }).unwrap())
			}
			Primitive::Url => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::iref::Iri }).unwrap())
			}
			Primitive::Uri => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::iref::Iri }).unwrap())
			}
			Primitive::Iri => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::iref::Iri }).unwrap())
			}
			Primitive::Cid => Ok(syn::parse2(quote! { ::treeldr_rust_prelude::ty::Cid }).unwrap()),
		}
	}
}