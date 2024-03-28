use iref::{IriBuf, IriRefBuf};
use serde::{Deserialize, Serialize};
use xsd_types::XSD_STRING;

use crate::abs::syntax::{Build, Context, BuildError, Scope};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CompactIri(pub IriRefBuf);

impl CompactIri {
	pub fn resolve(&self, scope: &Scope) -> Result<IriBuf, BuildError> {
		match self.0.as_iri() {
			Some(iri) => match scope.iri_prefix(iri.scheme().as_str()) {
				Some(prefix) => {
					let suffix = iri.split_once(':').unwrap().1;
					IriBuf::new(format!("{prefix}{suffix}")).map_err(|e| BuildError::InvalidIri(e.0))
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

impl From<IriBuf> for CompactIri {
	fn from(value: IriBuf) -> Self {
		Self(value.into())
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
