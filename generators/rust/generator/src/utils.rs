use core::fmt;
use iref::Iri;

pub fn ident_from_iri(iri: &Iri) -> Option<syn::Ident> {
	match iri.fragment() {
		Some(fragment) => syn::parse_str(PascalCase(fragment).to_string().as_str()).ok(),
		None => iri
			.path()
			.segments()
			.last()
			.and_then(|segment| syn::parse_str(PascalCase(segment).to_string().as_str()).ok()),
	}
}

pub struct PascalCase<T>(pub T);

impl<T: AsRef<str>> fmt::Display for PascalCase<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut upcase = true;

		for c in self.0.as_ref().chars() {
			if c.is_whitespace() || c.is_control() || c == '_' {
				// ignore.
				upcase = true
			} else if upcase {
				c.to_uppercase().fmt(f)?;
				upcase = false
			} else {
				c.fmt(f)?
			}
		}

		Ok(())
	}
}
