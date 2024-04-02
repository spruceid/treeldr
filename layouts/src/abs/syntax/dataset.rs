use json_syntax::TryFromJson;
use serde::{Deserialize, Serialize};

use super::{expect_array, Build, BuildError, Context, Error, Pattern, Scope};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Dataset(Vec<Quad>);

impl Dataset {
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

impl From<Vec<Quad>> for Dataset {
	fn from(value: Vec<Quad>) -> Self {
		Self(value)
	}
}

impl TryFromJson for Dataset {
	type Error = Error;

	fn try_from_json_at(
		json: &json_syntax::Value,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		Vec::try_from_json_at(json, code_map, offset).map(Self)
	}
}

impl<C: Context> Build<C> for Dataset
where
	C::Resource: Clone,
{
	type Target = crate::Dataset<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		let mut dataset = crate::Dataset::new();
		for quad in &self.0 {
			dataset.insert(quad.build(context, scope)?);
		}

		Ok(dataset)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Quad(
	pub Pattern,
	pub Pattern,
	pub Pattern,
	#[serde(default, skip_serializing_if = "Option::is_none")] pub Option<Pattern>,
);

impl TryFromJson for Quad {
	type Error = Error;

	fn try_from_json_at(
		json: &json_syntax::Value,
		code_map: &json_syntax::CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		let array = expect_array(json, offset)?;

		if array.len() < 3 {
			return Err(Error::MissingQuadPattern(offset));
		}

		if array.len() > 4 {
			return Err(Error::TooManyQuadPatterns(offset));
		}

		let mut component_offset = offset + 1;
		let s = Pattern::try_from_json_at(&array[0], code_map, component_offset)?;
		component_offset += code_map.get(component_offset).unwrap().volume;
		let p = Pattern::try_from_json_at(&array[1], code_map, component_offset)?;
		component_offset += code_map.get(component_offset).unwrap().volume;
		let o = Pattern::try_from_json_at(&array[2], code_map, component_offset)?;
		component_offset += code_map.get(component_offset).unwrap().volume;
		let g = array
			.get(3)
			.map(|g| Pattern::try_from_json_at(g, code_map, component_offset))
			.transpose()?;

		Ok(Self(s, p, o, g))
	}
}

impl<C: Context> Build<C> for Pattern {
	type Target = crate::Pattern<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		match self {
			Self::Var(name) => Ok(crate::Pattern::Var(scope.variable(name)?)),
			Self::Iri(compact_iri) => {
				let iri = compact_iri.resolve(scope)?;
				Ok(crate::Pattern::Resource(context.iri_resource(&iri)))
			}
			Self::Literal(l) => Ok(crate::Pattern::Resource(
				context.literal_resource(&l.value, l.type_.resolve(scope)?.as_lexical_type_ref()),
			)),
		}
	}
}

impl<C: Context> Build<C> for Quad {
	type Target = rdf_types::Quad<
		crate::Pattern<C::Resource>,
		crate::Pattern<C::Resource>,
		crate::Pattern<C::Resource>,
		crate::Pattern<C::Resource>,
	>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, BuildError> {
		Ok(rdf_types::Quad(
			self.0.build(context, scope)?,
			self.1.build(context, scope)?,
			self.2.build(context, scope)?,
			self.3
				.as_ref()
				.map(|g| g.build(context, scope))
				.transpose()?,
		))
	}
}
