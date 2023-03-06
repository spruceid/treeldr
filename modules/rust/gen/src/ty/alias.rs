use proc_macro2::Ident;
use treeldr::TId;

use super::Parameters;

#[derive(Debug)]
pub struct Alias {
	ident: Ident,
	target: TId<treeldr::Layout>,
	params: Parameters,
}

impl Alias {
	pub fn new(ident: Ident, target: TId<treeldr::Layout>) -> Self {
		Self {
			ident,
			target,
			params: Parameters::default(),
		}
	}

	pub fn ident(&self) -> &Ident {
		&self.ident
	}

	pub fn target(&self) -> TId<treeldr::Layout> {
		self.target
	}

	pub fn params(&self) -> Parameters {
		self.params
	}

	pub(crate) fn set_params(&mut self, p: Parameters) {
		self.params = p
	}

	pub(crate) fn compute_params(
		&self,
		mut dependency_params: impl FnMut(TId<treeldr::Layout>) -> Parameters,
	) -> Parameters {
		dependency_params(self.target)
	}
}
