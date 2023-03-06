use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug, Default, Clone, Copy)]
pub struct Parameters {
	/// Identifier type parameter.
	pub identifier: bool,
}

impl Parameters {
	pub fn identifier_parameter() -> Self {
		Self { identifier: true }
	}

	pub fn len(&self) -> usize {
		let mut l = 0;

		if self.identifier {
			l += 1
		}

		l
	}

	pub fn is_empty(&self) -> bool {
		!self.identifier
	}

	pub fn iter(&self) -> Iter {
		Iter {
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
		self.identifier |= other.identifier
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
	identifier: bool,
}

impl Iterator for Iter {
	type Item = Parameter;

	fn next(&mut self) -> Option<Self::Item> {
		if std::mem::take(&mut self.identifier) {
			return Some(Parameter::Identifier);
		}

		None
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Parameter {
	Identifier,
}

impl Parameter {
	pub fn default_value(&self) -> TokenStream {
		match self {
			Self::Identifier => quote!(I),
		}
	}
}

pub struct ParametersValues {
	identifier: TokenStream,
}

impl Default for ParametersValues {
	fn default() -> Self {
		Self::new(Parameter::Identifier.default_value())
	}
}

impl ParametersValues {
	pub fn new(identifier: TokenStream) -> Self {
		Self { identifier }
	}

	pub fn get(&self, p: Parameter) -> &TokenStream {
		match p {
			Parameter::Identifier => &self.identifier,
		}
	}
}

pub struct InstantiatedParameters<'a> {
	params: Parameters,
	values: &'a ParametersValues,
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
