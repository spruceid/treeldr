use crate::{vocab, Id, IriIndex, TId, MetaOption, FunctionalPropertyValue, prop::UnknownProperty};
use iref_enum::IriEnum;
use std::fmt;
use locspan::Meta;

pub mod restriction;

pub use crate::ty::data::RegExp;
pub use restriction::{
	WithRestrictions, WithRestrictionsIter,
};

pub trait RestrictionSet {
	fn is_restricted(&self) -> bool;
}

/// Primitive layout type.
pub trait PrimitiveLayoutType {
	type RestrictionRef<'a>;

	type Restrictions<M>: RestrictionSet;

	type RestrictionsIter<'a, M>
	where
		M: 'a;

	const PRIMITIVE: Primitive;
}

#[derive(IriEnum, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[iri_prefix("tldr" = "https://treeldr.org/")]
pub enum Primitive {
	/// Boolean.
	#[iri("tldr:Boolean")]
	Boolean,

	/// Integer number.
	#[iri("tldr:Integer")]
	Integer,

	/// Non negative integer number.
	#[iri("tldr:NonNegativeInteger")]
	NonNegativeInteger,

	/// Non positive integer number.
	#[iri("tldr:NonPositiveInteger")]
	NonPositiveInteger,

	/// Strictly negative integer number.
	#[iri("tldr:NegativeInteger")]
	NegativeInteger,

	/// Strictly positive integer number.
	#[iri("tldr:PositiveInteger")]
	PositiveInteger,

	/// Floating point number.
	#[iri("tldr:F32")]
	F32,

	/// Double.
	#[iri("tldr:F64")]
	F64,

	/// I64.
	#[iri("tldr:I64")]
	I64,

	/// I32.
	#[iri("tldr:I32")]
	I32,

	/// I16.
	#[iri("tldr:I16")]
	I16,

	/// I8.
	#[iri("tldr:I8")]
	I8,

	/// U64.
	#[iri("tldr:U64")]
	U64,

	/// U32.
	#[iri("tldr:U32")]
	U32,

	/// U16.
	#[iri("tldr:U16")]
	U16,

	/// U8.
	#[iri("tldr:U8")]
	U8,

	/// Base 64 byte string.
	#[iri("tldr:Base64Bytes")]
	Base64BytesBuf,

	/// Hex byte string.
	#[iri("tldr:HexBytes")]
	HexBytesBuf,

	/// String.
	#[iri("tldr:String")]
	String,

	/// Time.
	#[iri("tldr:Time")]
	Time,

	/// Date.
	#[iri("tldr:Date")]
	Date,

	/// Date and time.
	#[iri("tldr:DateTime")]
	DateTime,

	/// IRI.
	#[iri("tldr:IRI")]
	IriBuf,

	/// URI.
	#[iri("tldr:URI")]
	UriBuf,

	/// URL.
	#[iri("tldr:URL")]
	UrlBuf,

	/// Arbitrary bytes.
	#[iri("tldr:Bytes")]
	BytesBuf,

	/// CID (Content IDentifier).
	///
	/// See <https://github.com/multiformats/cid>
	#[iri("tldr:CID")]
	CidBuf,
}

impl Primitive {
	pub fn from_id(id: Id) -> Option<Self> {
		use vocab::{Term, TreeLdr};
		match id {
			Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Primitive(p)))) => Some(p),
			_ => None,
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Self::Boolean => "boolean",
			Self::Integer => "integer",
			Self::NonNegativeInteger => "non negative integer",
			Self::NonPositiveInteger => "non positive integer",
			Self::NegativeInteger => "negative integer",
			Self::PositiveInteger => "positive integer",
			Self::F32 => "f32",
			Self::F64 => "f64",
			Self::U64 => "u64",
			Self::U32 => "u32",
			Self::U16 => "u16",
			Self::U8 => "u8",
			Self::I64 => "i64",
			Self::I32 => "i32",
			Self::I16 => "i16",
			Self::I8 => "i8",
			Self::Base64BytesBuf => "base 64 bytes",
			Self::HexBytesBuf => "hex bytes",
			Self::String => "string",
			Self::Time => "time",
			Self::Date => "date",
			Self::DateTime => "date and time",
			Self::IriBuf => "iri",
			Self::UriBuf => "uri",
			Self::UrlBuf => "url",
			Self::BytesBuf => "bytes",
			Self::CidBuf => "content identifier",
		}
	}

	pub fn id(&self) -> Id {
		use vocab::{Term, TreeLdr};
		Id::Iri(IriIndex::Iri(Term::TreeLdr(TreeLdr::Primitive(*self))))
	}

	pub fn layout(&self) -> TId<crate::Layout> {
		TId::new(self.id())
	}

	pub fn natural_type_term(&self) -> Option<vocab::Term> {
		use vocab::{Term, Xsd};
		match self {
			Self::Boolean => Some(Term::Xsd(Xsd::Boolean)),
			Self::Integer => Some(Term::Xsd(Xsd::Integer)),
			Self::NonNegativeInteger => Some(Term::Xsd(Xsd::NonNegativeInteger)),
			Self::NonPositiveInteger => Some(Term::Xsd(Xsd::NonPositiveInteger)),
			Self::NegativeInteger => Some(Term::Xsd(Xsd::NegativeInteger)),
			Self::PositiveInteger => Some(Term::Xsd(Xsd::PositiveInteger)),
			Self::F32 => Some(Term::Xsd(Xsd::Float)),
			Self::F64 => Some(Term::Xsd(Xsd::Double)),
			Self::U64 => Some(Term::Xsd(Xsd::UnsignedLong)),
			Self::U32 => Some(Term::Xsd(Xsd::UnsignedInt)),
			Self::U16 => Some(Term::Xsd(Xsd::UnsignedShort)),
			Self::U8 => Some(Term::Xsd(Xsd::UnsignedByte)),
			Self::I64 => Some(Term::Xsd(Xsd::Long)),
			Self::I32 => Some(Term::Xsd(Xsd::Int)),
			Self::I16 => Some(Term::Xsd(Xsd::Short)),
			Self::I8 => Some(Term::Xsd(Xsd::Byte)),
			Self::Base64BytesBuf => Some(Term::Xsd(Xsd::Base64Binary)),
			Self::HexBytesBuf => Some(Term::Xsd(Xsd::HexBinary)),
			Self::String => Some(Term::Xsd(Xsd::String)),
			Self::Time => Some(Term::Xsd(Xsd::Time)),
			Self::Date => Some(Term::Xsd(Xsd::Date)),
			Self::DateTime => Some(Term::Xsd(Xsd::DateTime)),
			Self::IriBuf => Some(Term::Xsd(Xsd::AnyUri)),
			Self::UriBuf => Some(Term::Xsd(Xsd::AnyUri)),
			Self::UrlBuf => Some(Term::Xsd(Xsd::AnyUri)),
			Self::BytesBuf => None,
			Self::CidBuf => None,
		}
	}

	pub fn natural_type(&self) -> Option<TId<crate::Type>> {
		self.natural_type_term()
			.map(|t| TId::new(Id::Iri(IriIndex::Iri(t))))
	}
}

impl fmt::Display for Primitive {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.name().fmt(f)
	}
}

impl From<Primitive> for Id {
	fn from(value: Primitive) -> Self {
		value.id()
	}
}

#[derive(Clone, Debug)]
pub struct DerivedFrom<T: PrimitiveLayoutType, M> {
	restrictions: MetaOption<T::Restrictions<M>, M>,
	default: FunctionalPropertyValue<T, M>
}

impl<T: PrimitiveLayoutType, M> Default for DerivedFrom<T, M> {
	fn default() -> Self {
		Self {
			restrictions: MetaOption::default(),
			default: FunctionalPropertyValue::default()
		}
	}
}

impl<T: PrimitiveLayoutType, M> DerivedFrom<T, M> {
	pub fn is_restricted(&self) -> bool {
		self.restrictions.is_some_and(|r| r.is_restricted())
	}

	pub fn restrictions(&self) -> &MetaOption<T::Restrictions<M>, M> {
		&self.restrictions
	}

	pub fn default_value(&self) -> &FunctionalPropertyValue<T, M> {
		&self.default
	}
}

macro_rules! restricted_type {
	{ @lft $t:lifetime } => { 'a };
	{ $( $id:ident : $template:ident ),* } => {
		
		$(
			impl PrimitiveLayoutType for treeldr_primitives::$id {
				type RestrictionRef<'a> = <restriction::$template::Template as restriction::RestrictionsTemplate<Self>>::Ref<'a>;
	
				type Restrictions<M> = <restriction::$template::Template as restriction::RestrictionsTemplate<Self>>::Set<M>;
	
				type RestrictionsIter<'a, M> = <restriction::$template::Template as restriction::RestrictionsTemplate<Self>>::Iter<'a, M>
				where
					M: 'a;
	
				const PRIMITIVE: Primitive = Primitive::$id;
			}
		)*

		/// Derived primitive layout.
		#[derive(Clone, Debug)]
		pub enum Derived<M> {
			$(
				$id (DerivedFrom<treeldr_primitives::$id, M>),
			)*
		}

		impl<M> Derived<M> {
			pub fn primitive(&self) -> Primitive {
				match self {
					$(
						Self::$id(_) => <treeldr_primitives::$id as PrimitiveLayoutType>::PRIMITIVE,
					)*
				}
			}

			pub fn is_restricted(&self) -> bool {
				match self {
					$(
						Self::$id(r) => r.is_restricted(),
					)*
				}
			}

			pub fn restrictions(&self) -> Option<Meta<Restrictions<M>, &M>> {
				match self {
					$(
						Self::$id(r) => r.restrictions().as_ref().map(|m| m.borrow().map(Restrictions::$id)),
					)*
				}
			}

			pub fn with_restrictions(&self) -> Option<WithRestrictions<M>> {
				self.restrictions().and_then(WithRestrictions::new)
			}

			pub fn default_value(&self) -> DefaultValue<M> {
				match self {
					$(
						Self::$id(d) => DefaultValue::$id(d.default_value()),
					)*
				}
			}
		}

		impl<M> From<Primitive> for Derived<M> {
			fn from(p: Primitive) -> Self {
				match p {
					$(
						Primitive::$id => Self::$id(DerivedFrom::default()),
					)*
				}
			}
		}

		pub enum DefaultValue<'a, M: 'a> {
			$(
				$id(&'a FunctionalPropertyValue<treeldr_primitives::$id, M>),
			)*
		}

		impl<'a, M: 'a> DefaultValue<'a, M> {
			pub fn iter(&self) -> DefaultValueIter<'a, M> {
				match self {
					$(
						Self::$id(v) => DefaultValueIter::$id(v.iter()),
					)*
				}
			}
		}

		impl<'a, M: 'a> IntoIterator for DefaultValue<'a, M> {
			type Item = (Option<TId<UnknownProperty>>, Meta<crate::value::LiteralRef<'a>, &'a M>);
			type IntoIter = DefaultValueIter<'a, M>;

			fn into_iter(self) -> Self::IntoIter {
				self.iter()
			}
		}

		pub enum DefaultValueIter<'a, M: 'a> {
			$(
				$id(crate::property_values::functional::Iter<'a, treeldr_primitives::$id, M>),
			)*
		}

		impl<'a, M: 'a> Iterator for DefaultValueIter<'a, M> {
			type Item = (Option<TId<UnknownProperty>>, Meta<crate::value::LiteralRef<'a>, &'a M>);

			fn next(&mut self) -> Option<Self::Item> {
				todo!("default value to literal ref")
			}
		}

		#[derive(Clone, Copy)]
		pub enum RestrictionRef<'a> {
			$(
				$id(<treeldr_primitives::$id as PrimitiveLayoutType>::RestrictionRef<'a>),
			)*
		}

		pub enum Restrictions<'a, M: 'a> {
			$(
				$id(&'a <treeldr_primitives::$id as PrimitiveLayoutType>::Restrictions<M>),
			)*
		}

		impl<'a, M> Clone for Restrictions<'a, M> {
			fn clone(&self) -> Self {
				*self
			}
		}

		impl<'a, M> Copy for Restrictions<'a, M> {}

		impl<'a, M> Restrictions<'a, M> {
			pub fn is_restricted(&self) -> bool {
				match self {
					$(
						Self::$id(r) => r.is_restricted(),
					)*
				}
			}

			pub fn iter(&self) -> RestrictionsIter<'a, M> {
				match self {
					$(
						Self::$id(r) => RestrictionsIter::$id(r.iter()),
					)*
				}
			}
		}

		pub enum RestrictionsIter<'a, M: 'a> {
			None,
			$(
				$id(<treeldr_primitives::$id as PrimitiveLayoutType>::RestrictionsIter<'a, M>),
			)*
		}

		impl<'a, M> Default for RestrictionsIter<'a, M> {
			fn default() -> Self {
				Self::None
			}
		}

		impl<'a, M> Iterator for RestrictionsIter<'a, M> {
			type Item = Meta<RestrictionRef<'a>, &'a M>;

			fn next(&mut self) -> Option<Self::Item> {
				match self {
					Self::None => None,
					$(
						Self::$id(r) => r.next().map(|r| r.map(RestrictionRef::$id)),
					)*
				}
			}
		}

		impl<'a, M> DoubleEndedIterator for RestrictionsIter<'a, M> {
			fn next_back(&mut self) -> Option<Self::Item> {
				match self {
					Self::None => None,
					$(
						Self::$id(r) => r.next_back().map(|r| r.map(RestrictionRef::$id)),
					)*
				}
			}
		}
	};
}

restricted_type! {
	Boolean: none,
	Integer: integer,
	NonPositiveInteger: integer,
	NonNegativeInteger: integer,
	PositiveInteger: integer,
	NegativeInteger: integer,
	I64: integer,
	I32: integer,
	I16: integer,
	I8: integer,
	U64: integer,
	U32: integer,
	U16: integer,
	U8: integer,
	F32: float,
	F64: float,
	Base64BytesBuf: string,
	HexBytesBuf: string,
	String: unicode_string,
	Time: none,
	Date: none,
	DateTime: none,
	IriBuf: none,
	UriBuf: none,
	UrlBuf: none,
	BytesBuf: none,
	CidBuf: none
}