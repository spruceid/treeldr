use super::{
	alias::Alias, enumeration::Enum, structure::Struct, BuiltIn, Description, ParametersValues,
	Primitive, Type,
};
use crate::{doc_attribute, Context, Error, Generate, GenerateIn, Module, Referenced};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use rdf_types::Vocabulary;
use shelves::Ref;
use treeldr::{BlankIdIndex, IriIndex, TId};

mod json_ld;
mod rdf;
mod tr_impl;

impl<M> GenerateIn<M> for BuiltIn {
	fn generate_in<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		params: &ParametersValues,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		match self {
			Self::Required(item) => {
				item.generate_in(context, scope, params, tokens)?;
			}
			Self::Option(item) => {
				let item = item
					.generate_in_with(context, scope, params)
					.into_tokens()?;
				tokens.extend(quote! { Option<#item> })
			}
			Self::Vec(item) => {
				let item = item
					.generate_in_with(context, scope, params)
					.into_tokens()?;
				tokens.extend(quote! { Vec<#item> })
			}
			Self::BTreeSet(item) => {
				let item = item
					.generate_in_with(context, scope, params)
					.into_tokens()?;
				tokens.extend(quote! { std::collections::BTreeSet<#item> })
			}
			Self::BTreeMap(key, value) => {
				let key = key.generate_in_with(context, scope, params).into_tokens()?;
				let value = value
					.generate_in_with(context, scope, params)
					.into_tokens()?;
				tokens.extend(quote! { std::collections::BTreeMap<#key, #value> })
			}
			Self::OneOrMany(item) => {
				let item = item
					.generate_in_with(context, scope, params)
					.into_tokens()?;
				tokens.extend(quote! { ::treeldr_rust_prelude::OneOrMany<#item> })
			}
		}

		Ok(())
	}
}

impl<M> GenerateIn<M> for Referenced<BuiltIn> {
	fn generate_in<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		params: &ParametersValues,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		match self.0 {
			BuiltIn::Required(item) => {
				Referenced(item).generate_in(context, scope, params, tokens)?;
			}
			BuiltIn::Option(item) => {
				let item_ref = Referenced(item)
					.generate_in_with(context, scope, params)
					.into_tokens()?;
				tokens.extend(quote! { Option<#item_ref> })
			}
			BuiltIn::Vec(item) => {
				let item = item
					.generate_in_with(context, scope, params)
					.into_tokens()?;
				tokens.extend(quote! { &[#item] })
			}
			BuiltIn::BTreeSet(item) => {
				let item = item
					.generate_in_with(context, scope, params)
					.into_tokens()?;
				tokens.extend(quote! { &std::collections::BTreeSet<#item> })
			}
			BuiltIn::BTreeMap(key, value) => {
				let key = key.generate_in_with(context, scope, params).into_tokens()?;
				let value = value
					.generate_in_with(context, scope, params)
					.into_tokens()?;
				tokens.extend(quote! { &std::collections::BTreeMap<#key, #value> })
			}
			BuiltIn::OneOrMany(item) => {
				let item = item
					.generate_in_with(context, scope, params)
					.into_tokens()?;
				tokens.extend(quote! { &::treeldr_rust_prelude::OneOrMany<#item> })
			}
		}

		Ok(())
	}
}

impl<M> Generate<M> for Type {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let doc = doc_attribute(self.label(), self.documentation());
		tokens.extend(doc);
		match &self.desc {
			Description::Alias(a) => a.generate(context, scope, tokens),
			Description::Struct(s) => s.generate(context, scope, tokens),
			Description::Enum(e) => e.generate(context, scope, tokens),
			_ => Ok(()),
		}
	}
}

impl<M> Generate<M> for Alias {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let param_values = ParametersValues::default();
		let alias = self
			.target()
			.generate_in_with(context, scope, &param_values)
			.into_tokens()?;
		let ident = self.ident();
		let params = self.params().instantiate(&param_values);
		tokens.extend(quote! {
			pub type #ident #params = #alias #params ;
		});

		Ok(())
	}
}

impl<M> Generate<M> for Struct {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let ident = self.ident();
		let params_values = ParametersValues::default();
		let params = self.params().instantiate(&params_values);

		let mut fields = Vec::with_capacity(self.fields().len());
		let mut required_inputs = Vec::new();
		let mut fields_init = Vec::new();
		let mut derives = vec![
			quote! { Clone },
			quote! { PartialEq },
			quote! { Eq },
			quote! { PartialOrd },
			quote! { Ord },
			quote! { Debug },
		];

		for field in self.fields() {
			fields.push(
				field
					.generate_in_with(context, scope, &params_values)
					.into_tokens()?,
			);

			let field_ident = field.ident();
			let init = if field.ty(context).impl_default(context) {
				quote! {
					Default::default()
				}
			} else {
				let ty = field
					.layout()
					.generate_in_with(context, scope, &params_values)
					.into_tokens()?;
				required_inputs.push(quote! {
					#field_ident : #ty,
				});

				quote! {
					#field_ident
				}
			};

			fields_init.extend(quote! { #field_ident : #init, })
		}

		if required_inputs.is_empty() {
			derives.push(quote! { Default });
		}

		tokens.extend(quote! {
			#[derive(#(#derives),*)]
			pub struct #ident #params {
				#(#fields),*
			}
		});

		if !required_inputs.is_empty() {
			tokens.extend(quote! {
				impl #params #ident #params {
					pub fn new(#(#required_inputs)*) -> Self {
						Self {
							#(#fields_init)*
						}
					}
				}
			})
		}

		Ok(())
	}
}

impl<M> Generate<M> for Enum {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let ident = self.ident();
		let params_values = ParametersValues::default();
		let params = self.params().instantiate(&params_values);

		let mut variants = Vec::with_capacity(self.variants().len());

		for variant in self.variants() {
			variants.push(
				variant
					.generate_in_with(context, scope, &params_values)
					.into_tokens()?,
			)
		}

		tokens.extend(quote! {
			#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
			pub enum #ident #params {
				#(#variants),*
			}
		});

		Ok(())
	}
}

impl<M> GenerateIn<M> for TId<treeldr::Layout> {
	fn generate_in<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		params_values: &ParametersValues,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let ty = context
			.layout_type(*self)
			.expect("undefined generated layout");
		match &ty.desc {
			Description::Never => {
				tokens.extend(quote! { ! });
				Ok(())
			}
			Description::Primitive(p) => p.generate(context, scope, tokens),
			Description::Alias(a) => {
				let path = ty
					.path(context, a.ident().clone())
					.ok_or(Error::UnreachableType(*self))?;
				context.module_path(scope).to(&path).to_tokens(tokens);
				a.params().instantiate(params_values).to_tokens(tokens);
				Ok(())
			}
			Description::Struct(s) => {
				let path = ty
					.path(context, s.ident().clone())
					.ok_or(Error::UnreachableType(*self))?;
				context.module_path(scope).to(&path).to_tokens(tokens);
				s.params().instantiate(params_values).to_tokens(tokens);
				Ok(())
			}
			Description::Enum(e) => {
				let path = ty
					.path(context, e.ident().clone())
					.ok_or(Error::UnreachableType(*self))?;
				context.module_path(scope).to(&path).to_tokens(tokens);
				e.params().instantiate(params_values).to_tokens(tokens);
				Ok(())
			}
			Description::Reference(_) => {
				tokens.extend(quote! { ::treeldr_rust_prelude::Id<I> });
				Ok(())
			}
			Description::BuiltIn(b) => b.generate_in(context, scope, params_values, tokens),
		}
	}
}

impl<M> GenerateIn<M> for Referenced<TId<treeldr::Layout>> {
	fn generate_in<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		params_values: &ParametersValues,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let ty = context
			.layout_type(self.0)
			.expect("undefined generated layout");
		match &ty.desc {
			Description::Never => {
				tokens.extend(quote! { ! });
				Ok(())
			}
			Description::Primitive(p) => Referenced(*p).generate(context, scope, tokens),
			Description::Alias(a) => {
				let abs_path = ty
					.path(context, a.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?;
				let path = context.module_path(scope).to(&abs_path);
				let params = a.params().instantiate(params_values);
				tokens.extend(quote! { &#path #params });
				Ok(())
			}
			Description::Struct(s) => {
				let abs_path = ty
					.path(context, s.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?;
				let path = context.module_path(scope).to(&abs_path);
				let params = s.params().instantiate(params_values);
				tokens.extend(quote! { &#path #params });
				Ok(())
			}
			Description::Enum(e) => {
				let abs_path = ty
					.path(context, e.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?;
				let path = context.module_path(scope).to(&abs_path);
				let params = e.params().instantiate(params_values);
				tokens.extend(quote! { &#path #params });
				Ok(())
			}
			Description::Reference(_) => {
				tokens.extend(quote! { &::treeldr_rust_prelude::Id<I> });
				Ok(())
			}
			Description::BuiltIn(b) => {
				Referenced(*b).generate_in(context, scope, params_values, tokens)
			}
		}
	}
}

pub struct InContext<T>(pub T);

impl<M> GenerateIn<M> for InContext<TId<treeldr::Layout>> {
	fn generate_in<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		params_values: &ParametersValues,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		let ty = context
			.layout_type(self.0)
			.expect("undefined generated layout");
		match &ty.desc {
			Description::Never => {
				tokens.extend(quote! { ! });
				Ok(())
			}
			Description::Primitive(p) => p.generate(context, scope, tokens),
			Description::Alias(a) => {
				let path = ty
					.path(context, a.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?;
				context.module_path(scope).to(&path).to_tokens(tokens);
				a.params().instantiate(params_values).to_tokens(tokens);
				Ok(())
			}
			Description::Struct(s) => {
				let path = ty
					.path(context, s.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?;
				context.module_path(scope).to(&path).to_tokens(tokens);
				s.params().instantiate(params_values).to_tokens(tokens);
				Ok(())
			}
			Description::Enum(e) => {
				let path = ty
					.path(context, e.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?;
				context.module_path(scope).to(&path).to_tokens(tokens);
				e.params().instantiate(params_values).to_tokens(tokens);
				Ok(())
			}
			Description::Reference(ty_id) => {
				let tr = context.type_trait(*ty_id).unwrap();
				let context_path = context
					.module_path(scope)
					.to(&tr
						.context_path(context)
						.ok_or(Error::UnreachableTrait(*ty_id))?)
					.into_token_stream();
				let ident = tr.ident();
				let id_param_value = params_values.get(super::params::Parameter::Identifier);
				tokens.extend(quote! { <C as #context_path <#id_param_value>>::#ident });
				Ok(())
			}
			Description::BuiltIn(b) => b.generate_in(context, scope, params_values, tokens),
		}
	}
}

impl<M> Generate<M> for treeldr::layout::Primitive {
	fn generate<V>(
		&self,
		_context: &Context<V, M>,
		_scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		tokens.extend(match self {
			Self::Boolean => quote! { bool },
			Self::Integer => quote! { ::treeldr_rust_prelude::ty::Integer },
			Self::NonNegativeInteger => quote! { ::treeldr_rust_prelude::ty::NonNegativeInteger },
			Self::NonPositiveInteger => quote! { ::treeldr_rust_prelude::ty::NonPositiveInteger },
			Self::PositiveInteger => quote! { ::treeldr_rust_prelude::ty::PositiveInteger },
			Self::NegativeInteger => quote! { ::treeldr_rust_prelude::ty::NegativeInteger },
			Self::I64 => quote! { i64 },
			Self::I32 => quote! { i32 },
			Self::I16 => quote! { i16 },
			Self::I8 => quote! { i8 },
			Self::U64 => quote! { u64 },
			Self::U32 => quote! { u32 },
			Self::U16 => quote! { u16 },
			Self::U8 => quote! { u8 },
			Self::Float => quote! { f32 },
			Self::Double => quote! { f64 },
			Self::Base64Bytes => quote! { ::treeldr_rust_prelude::ty::Base64BytesBuf },
			Self::HexBytes => quote! { ::treeldr_rust_prelude::ty::HexBytesBuf },
			Self::String => quote! { ::std::string::String },
			Self::Date => quote! { ::treeldr_rust_prelude::chrono::NaiveDate },
			Self::DateTime => {
				quote! { ::treeldr_rust_prelude::chrono::DateTime<::treeldr_rust_prelude::chrono::Utc> }
			}
			Self::Time => quote! { ::treeldr_rust_prelude::chrono::NaiveTime },
			Self::Url => quote! { ::treeldr_rust_prelude::iref::IriBuf },
			Self::Uri => quote! { ::treeldr_rust_prelude::iref::IriBuf },
			Self::Iri => quote! { ::treeldr_rust_prelude::iref::IriBuf },
			Self::Bytes => quote! { ::treeldr_rust_prelude::ty::BytesBuf },
			Self::Cid => quote! { ::treeldr_rust_prelude::ty::CidBuf },
		});

		Ok(())
	}
}

impl<M> Generate<M> for Referenced<treeldr::layout::Primitive> {
	fn generate<V>(
		&self,
		_context: &Context<V, M>,
		_scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		tokens.extend(match self.0 {
			Primitive::Boolean => quote! { bool },
			Primitive::Integer => quote! { ::treeldr_rust_prelude::ty::Integer },
			Primitive::NonNegativeInteger => {
				quote! { &::treeldr_rust_prelude::ty::NonNegativeInteger }
			}
			Primitive::NonPositiveInteger => {
				quote! { &::treeldr_rust_prelude::ty::NonPositiveInteger }
			}
			Primitive::PositiveInteger => quote! { &::treeldr_rust_prelude::ty::PositiveInteger },
			Primitive::NegativeInteger => quote! { &::treeldr_rust_prelude::ty::NegativeInteger },
			Primitive::I64 => quote! { i64 },
			Primitive::I32 => quote! { i32 },
			Primitive::I16 => quote! { i16 },
			Primitive::I8 => quote! { i8 },
			Primitive::U64 => quote! { u64 },
			Primitive::U32 => quote! { u32 },
			Primitive::U16 => quote! { u16 },
			Primitive::U8 => quote! { u8 },
			Primitive::Float => quote! { f32 },
			Primitive::Double => quote! { f64 },
			Primitive::Base64Bytes => quote! { &::treeldr_rust_prelude::ty::Base64Bytes },
			Primitive::HexBytes => quote! { &::treeldr_rust_prelude::ty::HexBytes },
			Primitive::String => quote! { &str },
			Primitive::Date => quote! { ::treeldr_rust_prelude::chrono::NaiveDate },
			Primitive::DateTime => {
				quote! { ::treeldr_rust_prelude::chrono::DateTime<::treeldr_rust_prelude::chrono::Utc> }
			}
			Primitive::Time => quote! { ::treeldr_rust_prelude::chrono::NaiveTime },
			Primitive::Url => quote! { ::treeldr_rust_prelude::iref::Iri },
			Primitive::Uri => quote! { ::treeldr_rust_prelude::iref::Iri },
			Primitive::Iri => quote! { ::treeldr_rust_prelude::iref::Iri },
			Primitive::Bytes => quote! { &::treeldr_rust_prelude::ty::Bytes },
			Primitive::Cid => quote! { &::treeldr_rust_prelude::ty::Cid },
		});

		Ok(())
	}
}

pub trait GenerateFor<T, M> {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		ty: &T,
		tokens: &mut TokenStream,
	) -> Result<(), Error>;
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
