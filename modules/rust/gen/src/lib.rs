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
pub mod ty;

pub use context::Context;
pub use error::Error;
pub use module::Module;
pub use path::Path;
use treeldr::{BlankIdIndex, IriIndex};
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
	sep: &'a S
}

impl<'a, T, S, M> Generate<M> for SeparatedBy<'a, T, S>
where
	&'a T: IntoIterator,
	<&'a T as IntoIterator>::Item: Generate<M>,
	S: ToTokens
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