use super::{alias::Alias, BuiltIn, Description, Primitive, Type};
use crate::{syntax, Context, Error, GenerateSyntax, Referenced, Scope};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, IriIndex, TId};

// mod json_ld;
mod rdf;
mod tr_impl;

impl<M> GenerateSyntax<M> for BuiltIn {
	type Output = syn::Type;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		match self {
			Self::Required(item) => item.generate_syntax(context, scope),
			Self::Option(item) => {
				let item = item.generate_syntax(context, scope)?;
				Ok(syn::parse(quote!(Option<#item>).into()).unwrap())
			}
			Self::Vec(item) => {
				let item = item.generate_syntax(context, scope)?;
				Ok(syn::parse(quote!(Vec<#item>).into()).unwrap())
			}
			Self::BTreeSet(item) => {
				let item = item.generate_syntax(context, scope)?;
				Ok(syn::parse(quote!(std::collections::BTreeSet<#item>).into()).unwrap())
			}
			Self::BTreeMap(key, value) => {
				let key = key.generate_syntax(context, scope)?;
				let value = value.generate_syntax(context, scope)?;
				Ok(syn::parse(quote!(std::collections::BTreeMap<#key, #value>).into()).unwrap())
			}
			Self::OneOrMany(item) => {
				let item = item.generate_syntax(context, scope)?;
				Ok(syn::parse(quote!(::treeldr_rust_prelude::OneOrMany<#item>).into()).unwrap())
			}
		}
	}
}

impl<M> GenerateSyntax<M> for Referenced<BuiltIn> {
	type Output = syn::Type;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		match &self.0 {
			BuiltIn::Required(item) => Referenced(*item).generate_syntax(context, scope),
			BuiltIn::Option(item) => {
				let item = Referenced(*item).generate_syntax(context, scope)?;
				Ok(syn::parse(quote!(Option<#item>).into()).unwrap())
			}
			BuiltIn::Vec(item) => {
				let item = item.generate_syntax(context, scope)?;
				Ok(syn::parse(quote!(&[#item]).into()).unwrap())
			}
			BuiltIn::BTreeSet(item) => {
				let item = item.generate_syntax(context, scope)?;
				Ok(syn::parse(quote!(&std::collections::BTreeSet<#item>).into()).unwrap())
			}
			BuiltIn::BTreeMap(key, value) => {
				let key = key.generate_syntax(context, scope)?;
				let value = value.generate_syntax(context, scope)?;
				Ok(syn::parse(quote!(&std::collections::BTreeMap<#key, #value>).into()).unwrap())
			}
			BuiltIn::OneOrMany(item) => {
				let item = item.generate_syntax(context, scope)?;
				Ok(syn::parse(quote!(&::treeldr_rust_prelude::OneOrMany<#item>).into()).unwrap())
			}
		}
	}
}

impl<M> GenerateSyntax<M> for Type {
	type Output = Option<syntax::LayoutTypeDefinition>;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		match &self.desc {
			Description::Alias(a) => Ok(Some(syntax::LayoutTypeDefinition::Alias(
				a.generate_syntax(context, scope)?,
			))),
			Description::Struct(s) => Ok(Some(syntax::LayoutTypeDefinition::Struct(
				s.generate_syntax(context, scope)?,
			))),
			Description::Enum(e) => Ok(Some(syntax::LayoutTypeDefinition::Enum(
				e.generate_syntax(context, scope)?,
			))),
			_ => Ok(None),
		}
	}
}

impl<M> GenerateSyntax<M> for Alias {
	type Output = syntax::Alias;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		let params = syntax::LayoutParameters {
			identifier: if self.params().identifier {
				Some(format_ident!("I"))
			} else {
				None
			},
		};

		let mut scope = scope.clone();
		scope.params.identifier = params.identifier.clone().map(|i| {
			syn::Type::Path(syn::TypePath {
				qself: None,
				path: i.into(),
			})
		});

		let target = self.target().generate_syntax(context, &scope)?;

		Ok(syntax::Alias {
			ident: self.ident().clone(),
			params,
			target,
		})
	}
}

impl<M> GenerateSyntax<M> for TId<treeldr::Layout> {
	type Output = syn::Type;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		let ty = context
			.layout_type(*self)
			.expect("undefined generated layout");
		match &ty.desc {
			Description::Never => Ok(syn::parse2(quote!(!)).unwrap()),
			Description::Primitive(p) => p.generate_syntax(context, scope),
			Description::Alias(a) => {
				let path = ty
					.path(context, a.ident().clone())
					.ok_or(Error::UnreachableType(*self))?
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Struct(s) => {
				let path = ty
					.path(context, s.ident().clone())
					.ok_or(Error::UnreachableType(*self))?
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Enum(e) => {
				let path = ty
					.path(context, e.ident().clone())
					.ok_or(Error::UnreachableType(*self))?
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Reference(_) => {
				let id = scope
					.bound_params()
					.get(crate::ty::Parameter::Identifier)
					.unwrap();
				Ok(syn::parse2(quote!(::treeldr_rust_prelude::Id<#id>)).unwrap())
			}
			Description::BuiltIn(b) => b.generate_syntax(context, scope),
		}
	}
}

impl<M> GenerateSyntax<M> for Referenced<TId<treeldr::Layout>> {
	type Output = syn::Type;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		let ty = context
			.layout_type(self.0)
			.expect("undefined generated layout");
		match &ty.desc {
			Description::Never => Ok(syn::parse2(quote!(!)).unwrap()),
			Description::Primitive(p) => Referenced(*p).generate_syntax(context, scope),
			Description::Alias(a) => {
				let path = ty
					.path(context, a.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Struct(s) => {
				let path = ty
					.path(context, s.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Enum(e) => {
				let path = ty
					.path(context, e.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Reference(_) => {
				let id = scope
					.bound_params()
					.get(crate::ty::Parameter::Identifier)
					.unwrap();
				Ok(syn::parse2(quote!(&::treeldr_rust_prelude::Id<#id>)).unwrap())
			}
			Description::BuiltIn(b) => Referenced(*b).generate_syntax(context, scope),
		}
	}
}

pub struct InContext<T>(pub T);

impl<M> GenerateSyntax<M> for InContext<TId<treeldr::Layout>> {
	type Output = syn::Type;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		let ty = context
			.layout_type(self.0)
			.expect("undefined generated layout");
		match &ty.desc {
			Description::Never => Ok(syn::parse2(quote!(!)).unwrap()),
			Description::Primitive(p) => p.generate_syntax(context, scope),
			Description::Alias(a) => {
				let path = ty
					.path(context, a.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Struct(s) => {
				let path = ty
					.path(context, s.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Enum(e) => {
				let path = ty
					.path(context, e.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Reference(ty_id) => {
				let tr = context.type_trait(*ty_id).unwrap();
				let ident = tr.ident();
				let context_path = context
					.module_path(scope.module)
					.to(&tr
						.context_path(context)
						.ok_or(Error::UnreachableTrait(*ty_id))?)
					.generate_syntax(context, scope)?;
				let id_param_value = scope.bound_params().get(crate::ty::Parameter::Identifier);
				Ok(syn::parse2(quote! { <C as #context_path <#id_param_value>>::#ident }).unwrap())
			}
			Description::BuiltIn(b) => b.generate_syntax(context, scope),
		}
	}
}

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
			Self::Cid => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::ty::CidBuf }).unwrap())
			}
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
			Primitive::Cid => {
				Ok(syn::parse2(quote! { ::treeldr_rust_prelude::ty::Cid }).unwrap())
			}
		}
	}
}

pub fn type_ident_of_name(name: &treeldr::Name) -> proc_macro2::Ident {
	quote::format_ident!("{}", name.to_pascal_case())
}

pub fn field_ident_of_name(name: &treeldr::Name) -> proc_macro2::Ident {
	let mut name = name.to_snake_case();
	if matches!(name.as_str(), "type") {
		name.push('_')
	}

	quote::format_ident!("{}", name)
}

pub fn variant_ident_of_name(name: &treeldr::Name) -> proc_macro2::Ident {
	quote::format_ident!("{}", name.to_pascal_case())
}

pub fn doc_attribute(
	label: Option<&str>,
	doc: &treeldr::StrippedDocumentation,
) -> Vec<TokenStream> {
	let mut content = String::new();

	if let Some(label) = label {
		content.push_str(label)
	}

	if let Some(short) = doc.short_description() {
		if !content.is_empty() {
			content.push_str("\n\n");
		}

		content.push_str(short)
	}

	if let Some(long) = doc.long_description() {
		if !content.is_empty() {
			content.push_str("\n\n");
		}

		content.push_str(long)
	}

	content
		.lines()
		.map(|line| {
			quote::quote! {
				#[doc = #line]
			}
		})
		.collect()
}
