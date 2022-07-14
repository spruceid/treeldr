use proc_macro2::TokenStream;
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
pub use ty::Type;

pub trait Generate<F> {
	fn generate(
		&self,
		context: &Context<F>,
		scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>>;

	fn with<'a, 'c>(
		&self,
		context: &'c Context<'a, F>,
		scope: Option<Ref<Module<F>>>,
	) -> With<'a, 'c, '_, F, Self> {
		With(context, scope, self)
	}
}

pub struct With<'a, 'c, 't, F, T: ?Sized>(&'c Context<'a, F>, Option<Ref<Module<F>>>, &'t T);

impl<'a, 'c, 't, F, T: ?Sized + Generate<F>> With<'a, 'c, 't, F, T> {
	pub fn into_tokens(self) -> Result<TokenStream, Error<F>> {
		let mut tokens = TokenStream::new();
		self.2.generate(self.0, self.1, &mut tokens)?;
		Ok(tokens)
	}
}

pub struct Referenced<T>(T);
