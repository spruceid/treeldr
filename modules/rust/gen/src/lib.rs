use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::ToTokens;
use rdf_types::Vocabulary;
use thiserror::Error;

pub use shelves::Ref;

mod context;
mod error;
pub mod fmt;
pub mod module;
pub mod path;
pub mod tr;
pub mod ty;

pub use context::{Context, DedicatedSubModule, ModulePathBuilder, Options};
pub use error::Error;
pub use module::Module;
pub use path::Path;
use treeldr::{BlankIdIndex, IriIndex};
use ty::params::ParametersValues;
pub use ty::Type;

pub trait Generate<M> {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error>;

	fn generate_with<'a, 'c, V>(
		&self,
		context: &'c Context<'a, V, M>,
		scope: Option<Ref<Module>>,
	) -> With<'a, 'c, '_, V, M, Self> {
		With(context, scope, self)
	}
}

impl<'a, T: Generate<M>, M> Generate<M> for &'a T {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		(*self).generate(context, scope, tokens)
	}
}

pub trait GenerateIn<M> {
	fn generate_in<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		params: &ParametersValues,
		tokens: &mut TokenStream,
	) -> Result<(), Error>;

	fn generate_in_with<'a, 'c, 'p, V>(
		&self,
		context: &'c Context<'a, V, M>,
		scope: Option<Ref<Module>>,
		params: &'p ParametersValues,
	) -> InWith<'a, 'c, 'p, '_, V, M, Self> {
		InWith(context, scope, params, self)
	}
}

impl<'a, T: GenerateIn<M>, M> GenerateIn<M> for &'a T {
	fn generate_in<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		params: &ParametersValues,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		(*self).generate_in(context, scope, params, tokens)
	}
}

pub struct With<'a, 'c, 't, V, M, T: ?Sized>(&'c Context<'a, V, M>, Option<Ref<Module>>, &'t T);

impl<
		'a,
		'c,
		't,
		V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		M,
		T: ?Sized + Generate<M>,
	> With<'a, 'c, 't, V, M, T>
{
	pub fn into_tokens(self) -> Result<TokenStream, Error> {
		let mut tokens = TokenStream::new();
		self.2.generate(self.0, self.1, &mut tokens)?;
		Ok(tokens)
	}
}

pub struct InWith<'a, 'c, 'p, 't, V, M, T: ?Sized>(
	&'c Context<'a, V, M>,
	Option<Ref<Module>>,
	&'p ParametersValues,
	&'t T,
);

impl<
		'a,
		'c,
		'p,
		't,
		V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		M,
		T: ?Sized + GenerateIn<M>,
	> InWith<'a, 'c, 'p, 't, V, M, T>
{
	pub fn into_tokens(self) -> Result<TokenStream, Error> {
		let mut tokens = TokenStream::new();
		self.3.generate_in(self.0, self.1, self.2, &mut tokens)?;
		Ok(tokens)
	}
}

pub struct Referenced<T>(T);

pub trait GenerateList {
	fn separated_by<'a, S>(&'a self, sep: &'a S) -> SeparatedBy<'a, Self, S> {
		SeparatedBy { value: self, sep }
	}
}

impl<T> GenerateList for Vec<T> {}
impl<T> GenerateList for BTreeSet<T> {}

pub struct SeparatedBy<'a, T: ?Sized, S: ?Sized> {
	value: &'a T,
	sep: &'a S,
}

impl<'a, T, S, M> Generate<M> for SeparatedBy<'a, T, S>
where
	&'a T: IntoIterator,
	<&'a T as IntoIterator>::Item: Generate<M>,
	S: ToTokens,
{
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<Ref<Module>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error> {
		for i in self.value {
			i.generate(context, scope, tokens)?;
			self.sep.to_tokens(tokens);
		}

		Ok(())
	}
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
