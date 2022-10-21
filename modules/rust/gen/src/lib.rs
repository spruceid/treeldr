use proc_macro2::TokenStream;
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
		scope: Option<Ref<Module<M>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<M>>;

	fn with<'a, 'c, V>(
		&self,
		context: &'c Context<'a, V, M>,
		scope: Option<Ref<Module<M>>>,
	) -> With<'a, 'c, '_, V, M, Self> {
		With(context, scope, self)
	}
}

pub struct With<'a, 'c, 't, V, M, T: ?Sized>(&'c Context<'a, V, M>, Option<Ref<Module<M>>>, &'t T);

impl<
		'a,
		'c,
		't,
		V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>,
		M,
		T: ?Sized + Generate<M>,
	> With<'a, 'c, 't, V, M, T>
{
	pub fn into_tokens(self) -> Result<TokenStream, Error<M>> {
		let mut tokens = TokenStream::new();
		self.2.generate(self.0, self.1, &mut tokens)?;
		Ok(tokens)
	}
}

pub struct Referenced<T>(T);
