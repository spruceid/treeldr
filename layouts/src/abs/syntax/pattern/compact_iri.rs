use std::str::FromStr;

use iref::{IriBuf, IriRef, IriRefBuf};
use json_syntax::TryFromJsonSyntax;
use serde::{Deserialize, Serialize};
use xsd_types::XSD_STRING;

use crate::abs::syntax::{expect_string, Build, BuildError, Context, Error, Scope};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CompactIri(pub IriRefBuf);

impl CompactIri {
	pub fn resolve(&self, scope: &Scope) -> Result<IriBuf, BuildError> {
		match self.0.as_iri() {
			Some(iri) => match scope.iri_prefix(iri.scheme().as_str()) {
				Some(prefix) => {
					let suffix = iri.split_once(':').unwrap().1;
					IriBuf::new(format!("{prefix}{suffix}"))
						.map_err(|e| BuildError::InvalidIri(e.0))
				}
				None => Ok(iri.to_owned()),
			},
			None => match &scope.base_iri() {
				Some(base_iri) => Ok(self.0.resolved(base_iri)),
				None => Err(BuildError::NoBaseIri(self.0.clone())),
			},
		}
	}

	pub fn is_xsd_string(&self) -> bool {
		self.0 == XSD_STRING
	}

	pub fn xsd_string() -> Self {
		Self(XSD_STRING.to_owned().into())
	}
}

#[derive(Debug, thiserror::Error)]
#[error("invalid compact IRI `{0}`")]
pub struct InvalidCompactIri(pub String);

impl FromStr for CompactIri {
	type Err = InvalidCompactIri;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match IriRef::new(value) {
			Ok(iri_ref) => Ok(Self(iri_ref.to_owned())),
			Err(_) => Err(InvalidCompactIri(value.to_owned())),
		}
	}
}

impl From<IriBuf> for CompactIri {
	fn from(value: IriBuf) -> Self {
		Self(value.into())
	}
}

impl TryFromJsonSyntax for CompactIri {
	type Error = Error;

	fn try_from_json_syntax_at(
		json: &json_syntax::Value,
		_code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match IriRef::new(expect_string(json, offset)?) {
			Ok(iri_ref) => Ok(Self(iri_ref.to_owned())),
			Err(e) => Err(Error::InvalidCompactIri(offset, e.0.to_owned())),
		}
	}
}

impl<C: Context> Build<C> for CompactIri {
	type Target = C::Resource;

	/// Build this layout fragment using the given `context` in the given
	/// `scope`.
	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		let iri = self.resolve(scope)?;
		Ok(context.iri_resource(&iri))
	}
}
