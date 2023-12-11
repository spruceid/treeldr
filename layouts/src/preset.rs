use iref::Iri;
use static_iref::iri;

const ID_LAYOUT: &Iri = iri!("https://treeldr.org/layouts#id");
const UNIT_LAYOUT: &Iri = iri!("https://treeldr.org/layouts#unit");
const BOOLEAN_LAYOUT: &Iri = iri!("https://treeldr.org/layouts#boolean");
const U8_LAYOUT: &Iri = iri!("https://treeldr.org/layouts#u8");
const U16_LAYOUT: &Iri = iri!("https://treeldr.org/layouts#u16");
const U32_LAYOUT: &Iri = iri!("https://treeldr.org/layouts#u32");
const U64_LAYOUT: &Iri = iri!("https://treeldr.org/layouts#u64");
const I8_LAYOUT: &Iri = iri!("https://treeldr.org/layouts#i8");
const I16_LAYOUT: &Iri = iri!("https://treeldr.org/layouts#i16");
const I32_LAYOUT: &Iri = iri!("https://treeldr.org/layouts#i32");
const I64_LAYOUT: &Iri = iri!("https://treeldr.org/layouts#i64");
const STRING_LAYOUT: &Iri = iri!("https://treeldr.org/layouts#string");

pub enum PresetLayout {
	Id,
	Unit,
	Boolean,
	U8,
	U16,
	U32,
	U64,
	I8,
	I16,
	I32,
	I64,
	String,
}

impl PresetLayout {
	pub fn from_iri(iri: &Iri) -> Option<Self> {
		if iri == ID_LAYOUT {
			Some(Self::Id)
		} else if iri == UNIT_LAYOUT {
			Some(Self::Unit)
		} else if iri == BOOLEAN_LAYOUT {
			Some(Self::Boolean)
		} else if iri == U8_LAYOUT {
			Some(Self::U8)
		} else if iri == U16_LAYOUT {
			Some(Self::U16)
		} else if iri == U32_LAYOUT {
			Some(Self::U32)
		} else if iri == U64_LAYOUT {
			Some(Self::U64)
		} else if iri == I8_LAYOUT {
			Some(Self::I8)
		} else if iri == I16_LAYOUT {
			Some(Self::I16)
		} else if iri == I32_LAYOUT {
			Some(Self::I32)
		} else if iri == I64_LAYOUT {
			Some(Self::I64)
		} else if iri == STRING_LAYOUT {
			Some(Self::String)
		} else {
			None
		}
	}
}
