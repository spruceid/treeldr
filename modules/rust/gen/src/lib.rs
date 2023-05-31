use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use rdf_types::Vocabulary;
use thiserror::Error;

pub use shelves::Ref;

mod context;
mod error;
pub mod fmt;
pub mod module;
pub mod path;
pub mod syntax;
pub mod tr;
pub mod tr_impl;
pub mod ty;

pub use context::{Context, DedicatedSubModule, ModulePathBuilder, Options};
pub use error::Error;
pub use module::Module;
pub use path::Path;
use treeldr::{vocab, BlankIdIndex, Id, IriIndex, TId};
pub use ty::Type;

pub enum GenericArgumentRef<'a> {
	Lifetime(&'a syn::Lifetime),
	Type(&'a syn::Type),
}

impl<'a> GenericArgumentRef<'a> {
	pub fn into_owned(self) -> syn::GenericArgument {
		match self {
			Self::Lifetime(l) => syn::GenericArgument::Lifetime(l.clone()),
			Self::Type(t) => syn::GenericArgument::Type(t.clone()),
		}
	}
}

impl<'a> ToTokens for GenericArgumentRef<'a> {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			Self::Lifetime(l) => l.to_tokens(tokens),
			Self::Type(t) => t.to_tokens(tokens),
		}
	}
}

#[derive(Clone, Default)]
pub struct BoundParameters {
	pub lifetime: Option<syn::Lifetime>,
	pub identifier: Option<syn::Type>,
	pub context: Option<syn::Type>,
}

impl BoundParameters {
	pub fn get(&self, p: crate::ty::Parameter) -> Option<GenericArgumentRef> {
		match p {
			crate::ty::Parameter::Lifetime => {
				self.lifetime.as_ref().map(GenericArgumentRef::Lifetime)
			}
			crate::ty::Parameter::Identifier => {
				self.identifier.as_ref().map(GenericArgumentRef::Type)
			}
			crate::ty::Parameter::Context => self.context.as_ref().map(GenericArgumentRef::Type),
		}
	}

	pub fn add(
		&mut self,
		params: ty::Parameters,
		lifetime: impl FnOnce() -> syn::Lifetime,
		identifier: impl FnOnce() -> syn::Type,
		context: impl FnOnce() -> syn::Type,
	) {
		if params.context {
			self.context = Some(context())
		}

		if params.identifier {
			self.identifier = Some(identifier())
		}

		if params.lifetime {
			self.lifetime = Some(lifetime())
		}
	}
}

#[derive(Clone)]
pub struct Scope<'a> {
	pub module: Option<Ref<Module>>,
	pub params: BoundParameters,
	pub self_trait: Option<&'a tr::Trait>,
}

impl<'a> Scope<'a> {
	pub fn new(module: Option<Ref<Module>>) -> Self {
		Self {
			module,
			params: BoundParameters::default(),
			self_trait: None,
		}
	}

	pub fn bound_params(&self) -> &BoundParameters {
		&self.params
	}
}

pub trait GenerateSyntax<M> {
	type Output;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error>;
}

impl<M, T: GenerateSyntax<M>> GenerateSyntax<M> for Option<T> {
	type Output = Option<T::Output>;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		match self {
			Some(t) => Ok(Some(t.generate_syntax(context, scope)?)),
			None => Ok(None),
		}
	}
}

pub struct WithLayout<T> {
	value: T,
	layout: TId<treeldr::Layout>,
}

impl<T> WithLayout<T> {
	pub fn new(value: T, layout: TId<treeldr::Layout>) -> Self {
		Self { value, layout }
	}
}

impl<'a, M> GenerateSyntax<M> for WithLayout<&'a treeldr::value::Literal> {
	type Output = TokenStream;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		use treeldr::value::Literal;

		match (self.value, self.layout.id()) {
			(
				Literal::Boolean(b),
				Id::Iri(IriIndex::Iri(
					vocab::Term::Xsd(vocab::Xsd::Boolean)
					| vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(vocab::Primitive::Boolean)),
				)),
			) => Ok(quote!(#b)),
			(Literal::Numeric(n), _) => {
				WithLayout::new(n, self.layout).generate_syntax(context, scope)
			}
			(
				Literal::LangString(_),
				Id::Iri(IriIndex::Iri(vocab::Term::Rdf(vocab::Rdf::LangString))),
			) => todo!("generate rdf:LangString"),
			(
				Literal::String(s),
				Id::Iri(IriIndex::Iri(
					vocab::Term::Xsd(vocab::Xsd::String)
					| vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(vocab::Primitive::String)),
				)),
			) => Ok(quote!(#s.to_string())),
			(
				Literal::Base64Binary(b),
				Id::Iri(IriIndex::Iri(
					vocab::Term::Xsd(vocab::Xsd::Base64Binary)
					| vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
						vocab::Primitive::Base64BytesBuf,
					)),
				)),
			) => {
				let bytes = b.as_bytes();
				Ok(quote!(treeldr_rust_prelude::ty::Base64BinaryBuf::new(
					vec![#(#bytes),*]
				)))
			}
			(
				Literal::HexBinary(b),
				Id::Iri(IriIndex::Iri(
					vocab::Term::Xsd(vocab::Xsd::HexBinary)
					| vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(vocab::Primitive::HexBytesBuf)),
				)),
			) => {
				let bytes = b.as_bytes();
				Ok(quote!(treeldr_rust_prelude::ty::HexBinaryBuf::new(
					vec![#(#bytes),*]
				)))
			}
			(
				Literal::RegExp(_e),
				Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::RegularExpression))),
			) => {
				todo!("generate tldr:regularExpression")
			}
			(Literal::Other(_, _), _) => {
				todo!("generate other literal value")
			}
			_ => todo!("generate literal with unknown layout"),
		}
	}
}

impl<'a, M> GenerateSyntax<M> for WithLayout<&'a treeldr::value::Numeric> {
	type Output = TokenStream;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		use treeldr::value::Numeric;

		match (self.value, self.layout.id()) {
			(Numeric::Real(r), _) => {
				WithLayout::new(r, self.layout).generate_syntax(context, scope)
			}
			(Numeric::Float(f), Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::Float)))) => {
				let f = f.into_f32();
				Ok(quote!(#f))
			}
			(Numeric::Double(d), Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::Double)))) => {
				let d = d.into_f64();
				Ok(quote!(#d))
			}
			_ => todo!("generate literal with unknown layout"),
		}
	}
}

impl<'a, M> GenerateSyntax<M> for WithLayout<&'a treeldr::value::Real> {
	type Output = TokenStream;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		use treeldr::value::Real;

		match self.value {
			Real::Rational(r) => WithLayout::new(r, self.layout).generate_syntax(context, scope),
		}
	}
}

impl<'a, M> GenerateSyntax<M> for WithLayout<&'a treeldr::value::Rational> {
	type Output = TokenStream;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		_context: &Context<V, M>,
		_scope: &Scope,
	) -> Result<Self::Output, Error> {
		use treeldr::layout::Primitive;

		match self.layout.id() {
			Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::Integer)))
			| Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
				Primitive::Integer,
			)))) => {
				let bytes = self.value.as_integer().unwrap().to_signed_bytes_be();

				Ok(quote! {
					treeldr_rust_prelude::ty::Integer::from_signed_bytes_be(
						&[#(#bytes),*]
					)
				})
			}
			Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::NonPositiveInteger)))
			| Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
				Primitive::NonPositiveInteger,
			)))) => {
				let (_, bytes) = self
					.value
					.clone()
					.into_non_positive_integer()
					.unwrap()
					.to_bytes_be();

				Ok(quote! {
					treeldr_rust_prelude::ty::NonPositiveInteger::from_bytes_be(
						&[#(#bytes),*]
					)
				})
			}
			Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::NonNegativeInteger)))
			| Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
				Primitive::NonNegativeInteger,
			)))) => {
				let (_, bytes) = self
					.value
					.clone()
					.into_non_negative_integer()
					.unwrap()
					.to_bytes_be();

				Ok(quote! {
					treeldr_rust_prelude::ty::NonNegativeInteger::from_bytes_be(
						&[#(#bytes),*]
					)
				})
			}
			Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::PositiveInteger)))
			| Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
				Primitive::PositiveInteger,
			)))) => {
				let (_, bytes) = self
					.value
					.clone()
					.into_positive_integer()
					.unwrap()
					.to_bytes_be();

				Ok(quote! {
					treeldr_rust_prelude::ty::PositiveInteger::from_bytes_be(
						&[#(#bytes),*]
					)
				})
			}
			Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::NegativeInteger)))
			| Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
				Primitive::NegativeInteger,
			)))) => {
				let (_, bytes) = self
					.value
					.clone()
					.into_negative_integer()
					.unwrap()
					.to_bytes_be();

				Ok(quote! {
					treeldr_rust_prelude::ty::NonNegativeInteger::from_bytes_be(
						&[#(#bytes),*]
					)
				})
			}
			Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::UnsignedLong)))
			| Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
				Primitive::U64,
			)))) => {
				let value = self.value.clone().into_unsigned_long().unwrap();
				Ok(quote!(#value))
			}
			Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::UnsignedInt)))
			| Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
				Primitive::U32,
			)))) => {
				let value = self.value.clone().into_unsigned_int().unwrap();
				Ok(quote!(#value))
			}
			Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::UnsignedShort)))
			| Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
				Primitive::U16,
			)))) => {
				let value = self.value.clone().into_unsigned_short().unwrap();
				Ok(quote!(#value))
			}
			Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::UnsignedByte)))
			| Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
				Primitive::U8,
			)))) => {
				let value = self.value.clone().into_unsigned_byte().unwrap();
				Ok(quote!(#value))
			}
			Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::Long)))
			| Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
				Primitive::I64,
			)))) => {
				let value = self.value.clone().into_long().unwrap();
				Ok(quote!(#value))
			}
			Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::Int)))
			| Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
				Primitive::I32,
			)))) => {
				let value = self.value.clone().into_int().unwrap();
				Ok(quote!(#value))
			}
			Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::Short)))
			| Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
				Primitive::I16,
			)))) => {
				let value = self.value.clone().into_short().unwrap();
				Ok(quote!(#value))
			}
			Id::Iri(IriIndex::Iri(vocab::Term::Xsd(vocab::Xsd::Byte)))
			| Id::Iri(IriIndex::Iri(vocab::Term::TreeLdr(vocab::TreeLdr::Primitive(
				Primitive::I8,
			)))) => {
				let value = self.value.clone().into_byte().unwrap();
				Ok(quote!(#value))
			}
			_ => todo!("generate literal with unknown layout"),
		}
	}
}

// impl<M> GenerateSyntax<M> for tree {
// 	type Output = TokenStream;

// 	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
// 		&self,
// 		context: &Context<V, M>,
// 		scope: &Scope,
// 	) -> Result<Self::Output, Error> {
// 		match self {
// 			Self::Rational(r) => r.generate_syntax(context, scope)
// 		}
// 	}
// }

pub struct Referenced<T>(T);
