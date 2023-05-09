use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug, Default, Clone, Copy)]
pub struct Parameters {
	pub context: bool,

	/// Identifier type parameter.
	pub identifier: bool,
}

impl Parameters {
	pub fn identifier_parameter() -> Self {
		Self {
			context: false,
			identifier: true,
		}
	}

	pub fn context_parameter() -> Self {
		Self {
			context: true,
			identifier: false,
		}
	}

	pub fn with_context(self) -> Self {
		Self {
			context: true,
			..self
		}
	}

	pub fn len(&self) -> usize {
		let mut l = 0;

		if self.context {
			l += 1
		}

		if self.identifier {
			l += 1
		}

		l
	}

	pub fn is_empty(&self) -> bool {
		!self.context && !self.identifier
	}

	pub fn iter(&self) -> Iter {
		Iter {
			context: self.context,
			identifier: self.identifier,
		}
	}

	pub fn instantiate(self, values: &ParametersValues) -> InstantiatedParameters {
		InstantiatedParameters {
			params: self,
			values,
		}
	}

	pub fn append(&mut self, other: Self) {
		self.context |= other.context;
		self.identifier |= other.identifier
	}

	pub fn union_with(mut self, other: Self) -> Self {
		self.append(other);
		self
	}
}

impl IntoIterator for Parameters {
	type Item = Parameter;
	type IntoIter = Iter;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct Iter {
	context: bool,
	identifier: bool,
}

impl Iterator for Iter {
	type Item = Parameter;

	fn next(&mut self) -> Option<Self::Item> {
		if std::mem::take(&mut self.context) {
			return Some(Parameter::Context);
		}

		if std::mem::take(&mut self.identifier) {
			return Some(Parameter::Identifier);
		}

		None
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Parameter {
	Context,
	Identifier,
}

impl Parameter {
	pub fn default_value(&self) -> TokenStream {
		match self {
			Self::Context => quote!(C),
			Self::Identifier => quote!(I),
		}
	}
}

pub struct ParametersValues {
	context: Option<TokenStream>,
	identifier: Option<TokenStream>,
}

impl Default for ParametersValues {
	fn default() -> Self {
		Self::new(
			Parameter::Context.default_value(),
			Parameter::Identifier.default_value(),
		)
	}
}

impl ParametersValues {
	pub fn new(context: TokenStream, identifier: TokenStream) -> Self {
		Self {
			context: Some(context),
			identifier: Some(identifier),
		}
	}

	pub fn new_for_type(identifier: TokenStream) -> Self {
		Self {
			context: None,
			identifier: Some(identifier),
		}
	}

	pub fn new_for_trait(context: TokenStream) -> Self {
		Self {
			context: Some(context),
			identifier: None,
		}
	}

	pub fn get(&self, p: Parameter) -> Option<&TokenStream> {
		match p {
			Parameter::Context => self.context.as_ref(),
			Parameter::Identifier => self.identifier.as_ref(),
		}
	}
}

pub struct ParametersBounds {
	context: Option<TokenStream>,
	identifier: Option<TokenStream>,
}

impl Default for ParametersBounds {
	fn default() -> Self {
		Self::new(None, None)
	}
}

impl ParametersBounds {
	pub fn new(context: Option<TokenStream>, identifier: Option<TokenStream>) -> Self {
		Self {
			context,
			identifier,
		}
	}

	pub fn new_for_type(identifier: TokenStream) -> Self {
		Self {
			context: None,
			identifier: Some(identifier),
		}
	}

	pub fn new_for_trait(context: TokenStream) -> Self {
		Self {
			context: Some(context),
			identifier: None,
		}
	}

	pub fn get(&self, p: Parameter) -> Option<&TokenStream> {
		match p {
			Parameter::Context => self.context.as_ref(),
			Parameter::Identifier => self.identifier.as_ref(),
		}
	}
}

pub struct InstantiatedParameters<'a> {
	params: Parameters,
	values: &'a ParametersValues,
}

impl<'a> InstantiatedParameters<'a> {
	pub fn with_bounds(self, bounds: &'a ParametersBounds) -> BoundedInstantiatedParameters<'a> {
		BoundedInstantiatedParameters {
			params: self.params,
			values: self.values,
			bounds,
		}
	}
}

impl<'a> ToTokens for InstantiatedParameters<'a> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		if !self.params.is_empty() {
			tokens.extend(quote!(<));

			for (i, p) in self.params.into_iter().enumerate() {
				let v = self.values.get(p);
				if i == 0 {
					tokens.extend(quote!(#v));
				} else {
					tokens.extend(quote!(, #v));
				}
			}

			tokens.extend(quote!(>));
		}
	}
}

pub struct BoundedInstantiatedParameters<'a> {
	params: Parameters,
	values: &'a ParametersValues,
	bounds: &'a ParametersBounds,
}

impl<'a> ToTokens for BoundedInstantiatedParameters<'a> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		if !self.params.is_empty() {
			tokens.extend(quote!(<));

			for (i, p) in self.params.into_iter().enumerate() {
				let v = self.values.get(p);
				if i == 0 {
					tokens.extend(quote!(#v));
				} else {
					tokens.extend(quote!(, #v));
				}

				if let Some(bounds) = self.bounds.get(p) {
					tokens.extend(quote!(: #bounds))
				}
			}

			tokens.extend(quote!(>));
		}
	}
}
