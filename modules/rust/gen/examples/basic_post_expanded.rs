pub mod xsd {
	pub trait AnyDateTime<C: ?Sized> {}
	pub trait AnyDateTimeProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyDateTime>
	{
		type AnyDateTime: AnyDateTime<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyDateTime> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyDateTime>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyDateTime<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: AnyDateTime<C>> AnyDateTime<C> for &'r T {}
	pub trait AnyNonNegativeInteger<C: ?Sized> {}
	pub trait AnyNonNegativeIntegerProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyNonNegativeInteger>
	{
		type AnyNonNegativeInteger: AnyNonNegativeInteger<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyNonNegativeInteger> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyNonNegativeInteger>>::get(
				self, id,
			)
		}
	}
	impl<C: ?Sized> AnyNonNegativeInteger<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: AnyNonNegativeInteger<C>> AnyNonNegativeInteger<C> for &'r T {}
	pub trait UnsignedByte<C: ?Sized> {}
	pub trait UnsignedByteProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::UnsignedByte>
	{
		type UnsignedByte: UnsignedByte<Self>;
		fn get(&self, id: &I) -> Option<&Self::UnsignedByte> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::UnsignedByte>>::get(self, id)
		}
	}
	impl<C: ?Sized> UnsignedByte<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: UnsignedByte<C>> UnsignedByte<C> for &'r T {}
	pub trait Short<C: ?Sized> {}
	pub trait ShortProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Short> {
		type Short: Short<Self>;
		fn get(&self, id: &I) -> Option<&Self::Short> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Short>>::get(self, id)
		}
	}
	impl<C: ?Sized> Short<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Short<C>> Short<C> for &'r T {}
	pub trait PositiveInteger<C: ?Sized> {}
	pub trait PositiveIntegerProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::PositiveInteger>
	{
		type PositiveInteger: PositiveInteger<Self>;
		fn get(&self, id: &I) -> Option<&Self::PositiveInteger> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::PositiveInteger>>::get(self, id)
		}
	}
	impl<C: ?Sized> PositiveInteger<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: PositiveInteger<C>> PositiveInteger<C> for &'r T {}
	pub trait YearMonthDuration<C: ?Sized> {}
	pub trait YearMonthDurationProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::YearMonthDuration>
	{
		type YearMonthDuration: YearMonthDuration<Self>;
		fn get(&self, id: &I) -> Option<&Self::YearMonthDuration> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::YearMonthDuration>>::get(self, id)
		}
	}
	impl<C: ?Sized> YearMonthDuration<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: YearMonthDuration<C>> YearMonthDuration<C> for &'r T {}
	pub trait DayTimeDuration<C: ?Sized> {}
	pub trait DayTimeDurationProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::DayTimeDuration>
	{
		type DayTimeDuration: DayTimeDuration<Self>;
		fn get(&self, id: &I) -> Option<&Self::DayTimeDuration> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::DayTimeDuration>>::get(self, id)
		}
	}
	impl<C: ?Sized> DayTimeDuration<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: DayTimeDuration<C>> DayTimeDuration<C> for &'r T {}
	pub trait NegativeInteger<C: ?Sized> {}
	pub trait NegativeIntegerProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::NegativeInteger>
	{
		type NegativeInteger: NegativeInteger<Self>;
		fn get(&self, id: &I) -> Option<&Self::NegativeInteger> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::NegativeInteger>>::get(self, id)
		}
	}
	impl<C: ?Sized> NegativeInteger<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: NegativeInteger<C>> NegativeInteger<C> for &'r T {}
	pub trait Byte<C: ?Sized> {}
	pub trait ByteProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Byte> {
		type Byte: Byte<Self>;
		fn get(&self, id: &I) -> Option<&Self::Byte> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Byte>>::get(self, id)
		}
	}
	impl<C: ?Sized> Byte<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Byte<C>> Byte<C> for &'r T {}
	pub trait UnsignedLong<C: ?Sized> {}
	pub trait UnsignedLongProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::UnsignedLong>
	{
		type UnsignedLong: UnsignedLong<Self>;
		fn get(&self, id: &I) -> Option<&Self::UnsignedLong> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::UnsignedLong>>::get(self, id)
		}
	}
	impl<C: ?Sized> UnsignedLong<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: UnsignedLong<C>> UnsignedLong<C> for &'r T {}
	pub trait DateTimeStamp<C: ?Sized> {}
	pub trait DateTimeStampProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::DateTimeStamp>
	{
		type DateTimeStamp: DateTimeStamp<Self>;
		fn get(&self, id: &I) -> Option<&Self::DateTimeStamp> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::DateTimeStamp>>::get(self, id)
		}
	}
	impl<C: ?Sized> DateTimeStamp<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: DateTimeStamp<C>> DateTimeStamp<C> for &'r T {}
	pub trait AnyDate<C: ?Sized> {}
	pub trait AnyDateProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyDate>
	{
		type AnyDate: AnyDate<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyDate> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyDate>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyDate<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: AnyDate<C>> AnyDate<C> for &'r T {}
	pub trait AnyFloat<C: ?Sized> {}
	pub trait AnyFloatProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyFloat>
	{
		type AnyFloat: AnyFloat<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyFloat> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyFloat>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyFloat<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: AnyFloat<C>> AnyFloat<C> for &'r T {}
	pub trait NormalizedString<C: ?Sized> {}
	pub trait NormalizedStringProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::NormalizedString>
	{
		type NormalizedString: NormalizedString<Self>;
		fn get(&self, id: &I) -> Option<&Self::NormalizedString> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::NormalizedString>>::get(self, id)
		}
	}
	impl<C: ?Sized> NormalizedString<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: NormalizedString<C>> NormalizedString<C> for &'r T {}
	pub trait AnyString<C: ?Sized> {}
	pub trait AnyStringProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyString>
	{
		type AnyString: AnyString<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyString> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyString>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyString<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: AnyString<C>> AnyString<C> for &'r T {}
	pub trait AnyBoolean<C: ?Sized> {}
	pub trait AnyBooleanProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyBoolean>
	{
		type AnyBoolean: AnyBoolean<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyBoolean> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyBoolean>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyBoolean<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: AnyBoolean<C>> AnyBoolean<C> for &'r T {}
	pub trait Notation<C: ?Sized> {}
	pub trait NotationProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::Notation>
	{
		type Notation: Notation<Self>;
		fn get(&self, id: &I) -> Option<&Self::Notation> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Notation>>::get(self, id)
		}
	}
	impl<C: ?Sized> Notation<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Notation<C>> Notation<C> for &'r T {}
	pub trait Entity<C: ?Sized> {}
	pub trait EntityProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Entity> {
		type Entity: Entity<Self>;
		fn get(&self, id: &I) -> Option<&Self::Entity> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Entity>>::get(self, id)
		}
	}
	impl<C: ?Sized> Entity<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Entity<C>> Entity<C> for &'r T {}
	pub trait Entities<C: ?Sized> {}
	pub trait EntitiesProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::Entities>
	{
		type Entities: Entities<Self>;
		fn get(&self, id: &I) -> Option<&Self::Entities> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Entities>>::get(self, id)
		}
	}
	impl<C: ?Sized> Entities<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Entities<C>> Entities<C> for &'r T {}
	pub trait Long<C: ?Sized> {}
	pub trait LongProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Long> {
		type Long: Long<Self>;
		fn get(&self, id: &I) -> Option<&Self::Long> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Long>>::get(self, id)
		}
	}
	impl<C: ?Sized> Long<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Long<C>> Long<C> for &'r T {}
	pub trait Language<C: ?Sized> {}
	pub trait LanguageProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::Language>
	{
		type Language: Language<Self>;
		fn get(&self, id: &I) -> Option<&Self::Language> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Language>>::get(self, id)
		}
	}
	impl<C: ?Sized> Language<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Language<C>> Language<C> for &'r T {}
	pub trait Int<C: ?Sized> {}
	pub trait IntProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Int> {
		type Int: Int<Self>;
		fn get(&self, id: &I) -> Option<&Self::Int> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Int>>::get(self, id)
		}
	}
	impl<C: ?Sized> Int<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Int<C>> Int<C> for &'r T {}
	pub trait UnsignedInt<C: ?Sized> {}
	pub trait UnsignedIntProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::UnsignedInt>
	{
		type UnsignedInt: UnsignedInt<Self>;
		fn get(&self, id: &I) -> Option<&Self::UnsignedInt> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::UnsignedInt>>::get(self, id)
		}
	}
	impl<C: ?Sized> UnsignedInt<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: UnsignedInt<C>> UnsignedInt<C> for &'r T {}
	pub trait AnyAnyuri<C: ?Sized> {}
	pub trait AnyAnyuriProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyAnyuri>
	{
		type AnyAnyuri: AnyAnyuri<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyAnyuri> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyAnyuri>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyAnyuri<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: AnyAnyuri<C>> AnyAnyuri<C> for &'r T {}
	pub trait Qname<C: ?Sized> {}
	pub trait QnameProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Qname> {
		type Qname: Qname<Self>;
		fn get(&self, id: &I) -> Option<&Self::Qname> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Qname>>::get(self, id)
		}
	}
	impl<C: ?Sized> Qname<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Qname<C>> Qname<C> for &'r T {}
	pub trait AnyDecimal<C: ?Sized> {}
	pub trait AnyDecimalProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyDecimal>
	{
		type AnyDecimal: AnyDecimal<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyDecimal> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyDecimal>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyDecimal<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: AnyDecimal<C>> AnyDecimal<C> for &'r T {}
	pub trait Name<C: ?Sized> {}
	pub trait NameProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Name> {
		type Name: Name<Self>;
		fn get(&self, id: &I) -> Option<&Self::Name> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Name>>::get(self, id)
		}
	}
	impl<C: ?Sized> Name<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Name<C>> Name<C> for &'r T {}
	pub trait Ncname<C: ?Sized> {}
	pub trait NcnameProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Ncname> {
		type Ncname: Ncname<Self>;
		fn get(&self, id: &I) -> Option<&Self::Ncname> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Ncname>>::get(self, id)
		}
	}
	impl<C: ?Sized> Ncname<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Ncname<C>> Ncname<C> for &'r T {}
	pub trait Duration<C: ?Sized> {}
	pub trait DurationProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::Duration>
	{
		type Duration: Duration<Self>;
		fn get(&self, id: &I) -> Option<&Self::Duration> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Duration>>::get(self, id)
		}
	}
	impl<C: ?Sized> Duration<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Duration<C>> Duration<C> for &'r T {}
	pub trait AnyDouble<C: ?Sized> {}
	pub trait AnyDoubleProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyDouble>
	{
		type AnyDouble: AnyDouble<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyDouble> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyDouble>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyDouble<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: AnyDouble<C>> AnyDouble<C> for &'r T {}
	pub trait AnyTime<C: ?Sized> {}
	pub trait AnyTimeProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyTime>
	{
		type AnyTime: AnyTime<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyTime> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyTime>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyTime<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: AnyTime<C>> AnyTime<C> for &'r T {}
	pub trait GDay<C: ?Sized> {}
	pub trait GDayProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::GDay> {
		type GDay: GDay<Self>;
		fn get(&self, id: &I) -> Option<&Self::GDay> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::GDay>>::get(self, id)
		}
	}
	impl<C: ?Sized> GDay<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: GDay<C>> GDay<C> for &'r T {}
	pub trait Id<C: ?Sized> {}
	pub trait IdProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Id> {
		type Id: Id<Self>;
		fn get(&self, id: &I) -> Option<&Self::Id> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Id>>::get(self, id)
		}
	}
	impl<C: ?Sized> Id<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Id<C>> Id<C> for &'r T {}
	pub trait Idref<C: ?Sized> {}
	pub trait IdrefProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Idref> {
		type Idref: Idref<Self>;
		fn get(&self, id: &I) -> Option<&Self::Idref> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Idref>>::get(self, id)
		}
	}
	impl<C: ?Sized> Idref<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Idref<C>> Idref<C> for &'r T {}
	pub trait GYear<C: ?Sized> {}
	pub trait GYearProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::GYear> {
		type GYear: GYear<Self>;
		fn get(&self, id: &I) -> Option<&Self::GYear> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::GYear>>::get(self, id)
		}
	}
	impl<C: ?Sized> GYear<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: GYear<C>> GYear<C> for &'r T {}
	pub trait HexBinary<C: ?Sized> {}
	pub trait HexBinaryProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::HexBinary>
	{
		type HexBinary: HexBinary<Self>;
		fn get(&self, id: &I) -> Option<&Self::HexBinary> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::HexBinary>>::get(self, id)
		}
	}
	impl<C: ?Sized> HexBinary<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: HexBinary<C>> HexBinary<C> for &'r T {}
	pub trait Base64binary<C: ?Sized> {}
	pub trait Base64binaryProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::Base64binary>
	{
		type Base64binary: Base64binary<Self>;
		fn get(&self, id: &I) -> Option<&Self::Base64binary> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Base64binary>>::get(self, id)
		}
	}
	impl<C: ?Sized> Base64binary<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Base64binary<C>> Base64binary<C> for &'r T {}
	pub trait Token<C: ?Sized> {}
	pub trait TokenProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Token> {
		type Token: Token<Self>;
		fn get(&self, id: &I) -> Option<&Self::Token> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Token>>::get(self, id)
		}
	}
	impl<C: ?Sized> Token<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Token<C>> Token<C> for &'r T {}
	pub trait GMonth<C: ?Sized> {}
	pub trait GMonthProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::GMonth> {
		type GMonth: GMonth<Self>;
		fn get(&self, id: &I) -> Option<&Self::GMonth> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::GMonth>>::get(self, id)
		}
	}
	impl<C: ?Sized> GMonth<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: GMonth<C>> GMonth<C> for &'r T {}
	pub trait AnyInteger<C: ?Sized> {}
	pub trait AnyIntegerProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyInteger>
	{
		type AnyInteger: AnyInteger<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyInteger> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyInteger>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyInteger<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: AnyInteger<C>> AnyInteger<C> for &'r T {}
	pub trait NonPositiveInteger<C: ?Sized> {}
	pub trait NonPositiveIntegerProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::NonPositiveInteger>
	{
		type NonPositiveInteger: NonPositiveInteger<Self>;
		fn get(&self, id: &I) -> Option<&Self::NonPositiveInteger> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::NonPositiveInteger>>::get(self, id)
		}
	}
	impl<C: ?Sized> NonPositiveInteger<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: NonPositiveInteger<C>> NonPositiveInteger<C> for &'r T {}
	pub trait UnsignedShort<C: ?Sized> {}
	pub trait UnsignedShortProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::UnsignedShort>
	{
		type UnsignedShort: UnsignedShort<Self>;
		fn get(&self, id: &I) -> Option<&Self::UnsignedShort> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::UnsignedShort>>::get(self, id)
		}
	}
	impl<C: ?Sized> UnsignedShort<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: UnsignedShort<C>> UnsignedShort<C> for &'r T {}
	pub trait Idrefs<C: ?Sized> {}
	pub trait IdrefsProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Idrefs> {
		type Idrefs: Idrefs<Self>;
		fn get(&self, id: &I) -> Option<&Self::Idrefs> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Idrefs>>::get(self, id)
		}
	}
	impl<C: ?Sized> Idrefs<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Idrefs<C>> Idrefs<C> for &'r T {}
	pub trait Nmtokens<C: ?Sized> {}
	pub trait NmtokensProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::Nmtokens>
	{
		type Nmtokens: Nmtokens<Self>;
		fn get(&self, id: &I) -> Option<&Self::Nmtokens> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Nmtokens>>::get(self, id)
		}
	}
	impl<C: ?Sized> Nmtokens<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Nmtokens<C>> Nmtokens<C> for &'r T {}
	pub trait GMonthDay<C: ?Sized> {}
	pub trait GMonthDayProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::GMonthDay>
	{
		type GMonthDay: GMonthDay<Self>;
		fn get(&self, id: &I) -> Option<&Self::GMonthDay> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::GMonthDay>>::get(self, id)
		}
	}
	impl<C: ?Sized> GMonthDay<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: GMonthDay<C>> GMonthDay<C> for &'r T {}
	pub trait Nmtoken<C: ?Sized> {}
	pub trait NmtokenProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::Nmtoken>
	{
		type Nmtoken: Nmtoken<Self>;
		fn get(&self, id: &I) -> Option<&Self::Nmtoken> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Nmtoken>>::get(self, id)
		}
	}
	impl<C: ?Sized> Nmtoken<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Nmtoken<C>> Nmtoken<C> for &'r T {}
	pub type String = ::std::string::String;
	pub type Float = f32;
	pub type Integer = i32;
	pub type Anyuri = ::iref::IriBuf;
	pub type DateTime = ::chrono::DateTime<::chrono::Utc>;
	pub type Double = f64;
	pub type Time = ::chrono::NaiveTime;
	pub type Date = ::chrono::NaiveDate;
	pub type Boolean = bool;
	pub type Decimal = f64;
	pub type NonNegativeInteger = u32;
	impl<C: ?Sized> AnyTime<C> for ::chrono::NaiveTime {}
	impl<C: ?Sized> AnyBoolean<C> for bool {}
	impl<C: ?Sized> AnyNonNegativeInteger<C> for u32 {}
	impl<C: ?Sized> AnyAnyuri<C> for ::iref::IriBuf {}
	impl<C: ?Sized> AnyDateTime<C> for ::chrono::DateTime<::chrono::Utc> {}
	impl<C: ?Sized> AnyString<C> for ::std::string::String {}
	impl<C: ?Sized> AnyInteger<C> for i32 {}
	impl<C: ?Sized> AnyFloat<C> for f32 {}
	impl<C: ?Sized> AnyDouble<C> for f64 {}
	impl<C: ?Sized> AnyDate<C> for ::chrono::NaiveDate {}
	impl<C: ?Sized> AnyDecimal<C> for f64 {}
}
pub mod rebase {
	#[doc = " Terms of Use."]
	#[doc = ""]
	#[doc = "Tells the verifier what actions"]
	#[doc = " it is required to perform (an obligation), not allowed to perform"]
	#[doc = " (a prohibition), or allowed to perform (a permission) if it is to accept"]
	#[doc = " the verifiable credential or verifiable presentation."]
	#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
	pub struct TermOfUse {}
	#[doc = " Refreshing."]
	#[doc = ""]
	#[doc = "Provides enough information to the"]
	#[doc = " recipient's software such that the recipient can refresh the verifiable"]
	#[doc = " credential."]
	#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
	pub struct RefreshService {}
	#[doc = " Issuer."]
	#[doc = ""]
	#[doc = "It is RECOMMENDED that the URI in the issuer or its id be one which,"]
	#[doc = " if dereferenced, results in a document containing machine-readable"]
	#[doc = " information about the issuer that can be used to verify the information"]
	#[doc = " expressed in the credential."]
	#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
	pub enum Issuer<I> {
		Anyuri(super::xsd::Anyuri),
		Object(super::rdfs::Resource<I>),
	}
	#[doc = " Data Schema."]
	#[doc = ""]
	#[doc = "Provides verifiers with enough information"]
	#[doc = " to determine if the provided data conforms to the provided schema."]
	#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
	pub struct Schema {}
	#[doc = " Proof."]
	#[doc = ""]
	#[doc = "Cryptographic proof that can be used to detect tampering"]
	#[doc = " and verify the authorship of a credential or presentation."]
	#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
	pub struct Proof {}
	#[doc = " Evidence."]
	#[doc = ""]
	#[doc = "Evidence schemes providing enough information for a verifier"]
	#[doc = " to determine whether the evidence gathered by the issuer meets its"]
	#[doc = " confidence requirements for relying on the credential."]
	#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
	pub struct Evidence {}
	#[doc = " Status."]
	#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
	pub struct Status {}
	#[doc = " JSON-LD Frame for Verifiable Credential."]
	#[doc = ""]
	#[doc = "A set of one or more claims made by an issuer."]
	#[doc = " A verifiable credential is a tamper-evident credential that has authorship"]
	#[doc = " that can be cryptographically verified. Verifiable credentials can be used"]
	#[doc = " to build verifiable presentations, which can also be cryptographically"]
	#[doc = " verified. The claims in a credential can be about different subjects."]
	#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
	pub struct Credential<I> {
		pub expiration_date: Option<super::xsd::DateTime>,
		pub issuance_date: super::xsd::DateTime,
		pub credential_subject: std::collections::BTreeSet<super::rdfs::Resource<I>>,
		pub refresh_service: std::collections::BTreeSet<RefreshService>,
		pub credential_schema: std::collections::BTreeSet<Schema>,
		pub evidence: std::collections::BTreeSet<Evidence>,
		pub terms_of_use: std::collections::BTreeSet<TermOfUse>,
		pub credential_status: Option<Status>,
		pub proof: std::collections::BTreeSet<Proof>,
		pub issuer: Option<Issuer<I>>,
		#[doc = " Type of the credential."]
		pub type_: std::collections::BTreeSet<::treeldr_rust_prelude::Id<I>>,
		#[doc = " Identifier of the credential."]
		pub id: Option<::treeldr_rust_prelude::Id<I>>,
	}
	impl<I> Credential<I> {
		pub fn new(issuance_date: super::xsd::DateTime) -> Self {
			Self {
				expiration_date: Default::default(),
				issuance_date: issuance_date,
				credential_subject: Default::default(),
				refresh_service: Default::default(),
				credential_schema: Default::default(),
				evidence: Default::default(),
				terms_of_use: Default::default(),
				credential_status: Default::default(),
				proof: Default::default(),
				issuer: Default::default(),
				type_: Default::default(),
				id: Default::default(),
			}
		}
	}
	impl<C: ?Sized, I> super::rdfs::AnyResource<C> for Issuer<I> {
		type Comment;
		type Comments;
		type Type;
		type Types;
		type Label;
		type Labels;
		fn comment<'a>(&'a self, context: &'a C) {
			todo!()
		}
		fn type_<'a>(&'a self, context: &'a C) {
			todo!()
		}
		fn label<'a>(&'a self, context: &'a C) {
			todo!()
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N>
		for Issuer<N::Id>
	where
		N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
	{
		fn into_json_ld(self, namespace: &N) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
			todo!()
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N>
		for Credential<N::Id>
	where
		N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
	{
		fn into_json_ld(self, namespace: &N) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
			let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
			if let Some(value) = self.expiration_date {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("expirationDate".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			result.insert(
				::treeldr_rust_prelude::locspan::Meta("issuanceDate".into(), ()),
				::treeldr_rust_prelude::locspan::Meta(
					::treeldr_rust_prelude::IntoJsonLd::into_json_ld(self.issuance_date, namespace),
					(),
				),
			);
			if !self.credential_subject.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("credentialSubject".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.credential_subject
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if !self.refresh_service.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("refreshService".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.refresh_service
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if !self.credential_schema.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("credentialSchema".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.credential_schema
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if !self.evidence.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("evidence".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.evidence
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if !self.terms_of_use.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("termsOfUse".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.terms_of_use
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if let Some(value) = self.credential_status {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("credentialStatus".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			if !self.proof.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("proof".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.proof
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if let Some(value) = self.issuer {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("issuer".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			if !self.type_.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("type".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.type_
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if let Some(value) = self.id {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("id".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			result.into()
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
		for Schema
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		fn from_rdf<G>(
			namespace: &mut N,
			id: &N::Id,
			graph: &G,
		) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
		where
			G: ::treeldr_rust_prelude::grdf::Graph<
				Subject = N::Id,
				Predicate = N::Id,
				Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
			>,
		{
			Ok(Self {})
		}
	}
	impl<C: ?Sized> super::rdfs::AnyResource<C> for Status {
		type Comment < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Comments < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Type < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Types < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Label < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Labels < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
			::std::iter::empty()
		}
		fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
			::std::iter::empty()
		}
		fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
			::std::iter::empty()
		}
	}
	pub struct ProofTriplesAndValues<'a, I, V> {
		id_: Option<I>,
		_v: ::std::marker::PhantomData<&'a V>,
	}
	impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
		::treeldr_rust_prelude::RdfIterator<N> for ProofTriplesAndValues<'a, N::Id, V>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
		fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&mut self,
			vocabulary: &mut N,
			generator: &mut G,
		) -> Option<Self::Item> {
			self.id_
				.take()
				.map(::treeldr_rust_prelude::rdf_types::Object::Id)
				.map(::treeldr_rust_prelude::rdf::TripleOrValue::Value)
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
		::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for Proof
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type TriplesAndValues < 'a > = ProofTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
		fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&'a self,
			namespace: &mut N,
			generator: &mut G,
		) -> Self::TriplesAndValues<'a>
		where
			N::Id: 'a,
			V: 'a,
		{
			ProofTriplesAndValues {
				id_: Some(generator.next(namespace)),
				_v: ::std::marker::PhantomData,
			}
		}
	}
	impl<C: ?Sized> super::rdfs::AnyResource<C> for RefreshService {
		type Comment < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Comments < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Type < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Types < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Label < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Labels < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
			::std::iter::empty()
		}
		fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
			::std::iter::empty()
		}
		fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
			::std::iter::empty()
		}
	}
	impl<C: ?Sized> super::rdfs::AnyResource<C> for Evidence {
		type Comment < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Comments < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Type < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Types < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Label < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Labels < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
			::std::iter::empty()
		}
		fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
			::std::iter::empty()
		}
		fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
			::std::iter::empty()
		}
	}
	pub struct CredentialTriplesAndValues<'a, I, V> {
		id_: Option<I>,
		expiration_date: ::treeldr_rust_prelude::rdf::iter::Optional<
			::treeldr_rust_prelude::rdf::ValuesOnly<
				::treeldr_rust_prelude::rdf::LiteralValue<
					'a,
					::chrono::DateTime<::chrono::Utc>,
					I,
					V,
				>,
			>,
		>,
		issuance_date: ::treeldr_rust_prelude::rdf::ValuesOnly<
			::treeldr_rust_prelude::rdf::LiteralValue<'a, ::chrono::DateTime<::chrono::Utc>, I, V>,
		>,
		credential_subject: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, super::rdfs::Resource<I>>,
			super::rdfs::ResourceTriplesAndValues<'a, I, V>,
			V,
		>,
		refresh_service: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, RefreshService>,
			RefreshServiceTriplesAndValues<'a, I, V>,
			V,
		>,
		credential_schema: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, Schema>,
			SchemaTriplesAndValues<'a, I, V>,
			V,
		>,
		evidence: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, Evidence>,
			EvidenceTriplesAndValues<'a, I, V>,
			V,
		>,
		terms_of_use: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, TermOfUse>,
			TermOfUseTriplesAndValues<'a, I, V>,
			V,
		>,
		credential_status:
			::treeldr_rust_prelude::rdf::iter::Optional<StatusTriplesAndValues<'a, I, V>>,
		proof: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, Proof>,
			ProofTriplesAndValues<'a, I, V>,
			V,
		>,
		issuer: ::treeldr_rust_prelude::rdf::iter::Optional<IssuerTriplesAndValues<'a, I, V>>,
		type_: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, ::treeldr_rust_prelude::Id<I>>,
			::treeldr_rust_prelude::rdf::ValuesOnly<::treeldr_rust_prelude::rdf::IdValue<'a, I, V>>,
			V,
		>,
	}
	impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
		::treeldr_rust_prelude::RdfIterator<N> for CredentialTriplesAndValues<'a, N::Id, V>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::chrono::DateTime<::chrono::Utc>: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
		::iref::IriBuf: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
	{
		type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
		fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&mut self,
			vocabulary: &mut N,
			generator: &mut G,
		) -> Option<Self::Item> {
			self . type_ . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"))) , value)) } }) . or_else (|| self . issuer . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#issuer"))) , value)) } }) . or_else (|| self . proof . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#proof"))) , value)) } }) . or_else (|| self . credential_status . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#credentialStatus"))) , value)) } }) . or_else (|| self . terms_of_use . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#termsOfUse"))) , value)) } }) . or_else (|| self . evidence . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#evidence"))) , value)) } }) . or_else (|| self . credential_schema . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#credentialSchema"))) , value)) } }) . or_else (|| self . refresh_service . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#refreshService"))) , value)) } }) . or_else (|| self . credential_subject . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#credentialSubject"))) , value)) } }) . or_else (|| self . issuance_date . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#issuanceDate"))) , value)) } }) . or_else (|| self . expiration_date . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#expirationDate"))) , value)) } }) . or_else (|| self . id_ . take () . map (:: treeldr_rust_prelude :: rdf_types :: Object :: Id) . map (:: treeldr_rust_prelude :: rdf :: TripleOrValue :: Value))))))))))))
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
		::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for Credential<N::Id>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::chrono::DateTime<::chrono::Utc>: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
		::iref::IriBuf: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
	{
		type TriplesAndValues < 'a > = CredentialTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
		fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&'a self,
			namespace: &mut N,
			generator: &mut G,
		) -> Self::TriplesAndValues<'a>
		where
			N::Id: 'a,
			V: 'a,
		{
			CredentialTriplesAndValues {
				id_: Some(
					self.id
						.clone()
						.map(::treeldr_rust_prelude::Id::unwrap)
						.unwrap_or_else(|| generator.next(namespace)),
				),
				expiration_date: self
					.expiration_date
					.unbound_rdf_triples_and_values(namespace, generator),
				issuance_date: self
					.issuance_date
					.unbound_rdf_triples_and_values(namespace, generator),
				credential_subject: self
					.credential_subject
					.unbound_rdf_triples_and_values(namespace, generator),
				refresh_service: self
					.refresh_service
					.unbound_rdf_triples_and_values(namespace, generator),
				credential_schema: self
					.credential_schema
					.unbound_rdf_triples_and_values(namespace, generator),
				evidence: self
					.evidence
					.unbound_rdf_triples_and_values(namespace, generator),
				terms_of_use: self
					.terms_of_use
					.unbound_rdf_triples_and_values(namespace, generator),
				credential_status: self
					.credential_status
					.unbound_rdf_triples_and_values(namespace, generator),
				proof: self
					.proof
					.unbound_rdf_triples_and_values(namespace, generator),
				issuer: self
					.issuer
					.unbound_rdf_triples_and_values(namespace, generator),
				type_: self
					.type_
					.unbound_rdf_triples_and_values(namespace, generator),
			}
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N>
		for Proof
	where
		N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
	{
		fn into_json_ld(self, namespace: &N) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
			let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
			result.into()
		}
	}
	impl<C: ?Sized> super::rdfs::AnyResource<C> for Schema {
		type Comment < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Comments < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Type < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Types < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Label < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Labels < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
			::std::iter::empty()
		}
		fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
			::std::iter::empty()
		}
		fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
			::std::iter::empty()
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
		for RefreshService
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		fn from_rdf<G>(
			namespace: &mut N,
			id: &N::Id,
			graph: &G,
		) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
		where
			G: ::treeldr_rust_prelude::grdf::Graph<
				Subject = N::Id,
				Predicate = N::Id,
				Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
			>,
		{
			Ok(Self {})
		}
	}
	impl<C: ?Sized> super::rdfs::AnyResource<C> for Proof {
		type Comment < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Comments < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Type < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Types < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Label < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Labels < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
			::std::iter::empty()
		}
		fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
			::std::iter::empty()
		}
		fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
			::std::iter::empty()
		}
	}
	pub struct RefreshServiceTriplesAndValues<'a, I, V> {
		id_: Option<I>,
		_v: ::std::marker::PhantomData<&'a V>,
	}
	impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
		::treeldr_rust_prelude::RdfIterator<N> for RefreshServiceTriplesAndValues<'a, N::Id, V>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
		fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&mut self,
			vocabulary: &mut N,
			generator: &mut G,
		) -> Option<Self::Item> {
			self.id_
				.take()
				.map(::treeldr_rust_prelude::rdf_types::Object::Id)
				.map(::treeldr_rust_prelude::rdf::TripleOrValue::Value)
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
		::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for RefreshService
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type TriplesAndValues < 'a > = RefreshServiceTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
		fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&'a self,
			namespace: &mut N,
			generator: &mut G,
		) -> Self::TriplesAndValues<'a>
		where
			N::Id: 'a,
			V: 'a,
		{
			RefreshServiceTriplesAndValues {
				id_: Some(generator.next(namespace)),
				_v: ::std::marker::PhantomData,
			}
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
		for Evidence
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		fn from_rdf<G>(
			namespace: &mut N,
			id: &N::Id,
			graph: &G,
		) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
		where
			G: ::treeldr_rust_prelude::grdf::Graph<
				Subject = N::Id,
				Predicate = N::Id,
				Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
			>,
		{
			Ok(Self {})
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N>
		for Schema
	where
		N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
	{
		fn into_json_ld(self, namespace: &N) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
			let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
			result.into()
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N>
		for Status
	where
		N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
	{
		fn into_json_ld(self, namespace: &N) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
			let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
			result.into()
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
		for Issuer<N::Id>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		fn from_rdf<G>(
			namespace: &mut N,
			id: &N::Id,
			graph: &G,
		) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
		where
			G: ::treeldr_rust_prelude::grdf::Graph<
				Subject = N::Id,
				Predicate = N::Id,
				Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
			>,
		{
			todo!()
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
		for Proof
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		fn from_rdf<G>(
			namespace: &mut N,
			id: &N::Id,
			graph: &G,
		) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
		where
			G: ::treeldr_rust_prelude::grdf::Graph<
				Subject = N::Id,
				Predicate = N::Id,
				Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
			>,
		{
			Ok(Self {})
		}
	}
	impl<C: ?Sized + super::rdfs::AnyResourceProvider<I> + super::rdfs::ClassProvider<I>, I>
		super::rdfs::AnyResource<C> for Credential<I>
	{
		type Comment < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Comments < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Type < 'a > = & 'a < C as super :: rdfs :: ClassProvider < I >> :: Class where Self : 'a , C : 'a ;
		type Types < 'a > = :: treeldr_rust_prelude :: iter :: Fetch < 'a , C , < C as super :: rdfs :: ClassProvider < I >> :: Class , :: std :: collections :: btree_set :: Iter < 'a , :: treeldr_rust_prelude :: Id < I > > > where Self : 'a , C : 'a ;
		type Label < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Labels < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
			::std::iter::empty()
		}
		fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
			::treeldr_rust_prelude::iter::Fetch::new(context, self.type_.iter())
		}
		fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
			::std::iter::empty()
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N>
		for TermOfUse
	where
		N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
	{
		fn into_json_ld(self, namespace: &N) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
			let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
			result.into()
		}
	}
	pub struct TermOfUseTriplesAndValues<'a, I, V> {
		id_: Option<I>,
		_v: ::std::marker::PhantomData<&'a V>,
	}
	impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
		::treeldr_rust_prelude::RdfIterator<N> for TermOfUseTriplesAndValues<'a, N::Id, V>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
		fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&mut self,
			vocabulary: &mut N,
			generator: &mut G,
		) -> Option<Self::Item> {
			self.id_
				.take()
				.map(::treeldr_rust_prelude::rdf_types::Object::Id)
				.map(::treeldr_rust_prelude::rdf::TripleOrValue::Value)
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
		::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for TermOfUse
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type TriplesAndValues < 'a > = TermOfUseTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
		fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&'a self,
			namespace: &mut N,
			generator: &mut G,
		) -> Self::TriplesAndValues<'a>
		where
			N::Id: 'a,
			V: 'a,
		{
			TermOfUseTriplesAndValues {
				id_: Some(generator.next(namespace)),
				_v: ::std::marker::PhantomData,
			}
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N>
		for Evidence
	where
		N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
	{
		fn into_json_ld(self, namespace: &N) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
			let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
			result.into()
		}
	}
	pub struct SchemaTriplesAndValues<'a, I, V> {
		id_: Option<I>,
		_v: ::std::marker::PhantomData<&'a V>,
	}
	impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
		::treeldr_rust_prelude::RdfIterator<N> for SchemaTriplesAndValues<'a, N::Id, V>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
		fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&mut self,
			vocabulary: &mut N,
			generator: &mut G,
		) -> Option<Self::Item> {
			self.id_
				.take()
				.map(::treeldr_rust_prelude::rdf_types::Object::Id)
				.map(::treeldr_rust_prelude::rdf::TripleOrValue::Value)
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
		::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for Schema
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type TriplesAndValues < 'a > = SchemaTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
		fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&'a self,
			namespace: &mut N,
			generator: &mut G,
		) -> Self::TriplesAndValues<'a>
		where
			N::Id: 'a,
			V: 'a,
		{
			SchemaTriplesAndValues {
				id_: Some(generator.next(namespace)),
				_v: ::std::marker::PhantomData,
			}
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
		for Status
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		fn from_rdf<G>(
			namespace: &mut N,
			id: &N::Id,
			graph: &G,
		) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
		where
			G: ::treeldr_rust_prelude::grdf::Graph<
				Subject = N::Id,
				Predicate = N::Id,
				Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
			>,
		{
			Ok(Self {})
		}
	}
	pub struct StatusTriplesAndValues<'a, I, V> {
		id_: Option<I>,
		_v: ::std::marker::PhantomData<&'a V>,
	}
	impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
		::treeldr_rust_prelude::RdfIterator<N> for StatusTriplesAndValues<'a, N::Id, V>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
		fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&mut self,
			vocabulary: &mut N,
			generator: &mut G,
		) -> Option<Self::Item> {
			self.id_
				.take()
				.map(::treeldr_rust_prelude::rdf_types::Object::Id)
				.map(::treeldr_rust_prelude::rdf::TripleOrValue::Value)
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
		::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for Status
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type TriplesAndValues < 'a > = StatusTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
		fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&'a self,
			namespace: &mut N,
			generator: &mut G,
		) -> Self::TriplesAndValues<'a>
		where
			N::Id: 'a,
			V: 'a,
		{
			StatusTriplesAndValues {
				id_: Some(generator.next(namespace)),
				_v: ::std::marker::PhantomData,
			}
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
		for Credential<N::Id>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::chrono::DateTime<::chrono::Utc>: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>,
		::iref::IriBuf: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>,
	{
		fn from_rdf<G>(
			namespace: &mut N,
			id: &N::Id,
			graph: &G,
		) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
		where
			G: ::treeldr_rust_prelude::grdf::Graph<
				Subject = N::Id,
				Predicate = N::Id,
				Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
			>,
		{
			Ok(Self {
				expiration_date: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#expirationDate"
							),
						)),
					);
					let object = objects.next();
					if objects.next().is_some() {
						panic!("multiples values on functional property")
					}
					match object {
						Some(object) => Some({
							match object { :: treeldr_rust_prelude :: rdf :: Object :: Literal (lit) => { < :: chrono :: DateTime < :: chrono :: Utc > as :: treeldr_rust_prelude :: rdf :: FromLiteral < V , N >> :: from_literal (namespace , lit) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: ExpectedLiteralValue) }
						}),
						None => None,
					}
				},
				issuance_date: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#issuanceDate"
							),
						)),
					);
					match objects.next() {
						Some(object) => {
							if objects.next().is_some() {
								panic!("multiples values on functional property")
							}
							match object { :: treeldr_rust_prelude :: rdf :: Object :: Literal (lit) => { < :: chrono :: DateTime < :: chrono :: Utc > as :: treeldr_rust_prelude :: rdf :: FromLiteral < V , N >> :: from_literal (namespace , lit) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: ExpectedLiteralValue) }
						}
						None => {
							return Err(
								::treeldr_rust_prelude::FromRdfError::MissingRequiredPropertyValue,
							)
						}
					}
				},
				credential_subject: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#credentialSubject"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				refresh_service: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#refreshService"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				credential_schema: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#credentialSchema"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				evidence: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#evidence"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				terms_of_use: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#termsOfUse"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				credential_status: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#credentialStatus"
							),
						)),
					);
					let object = objects.next();
					if objects.next().is_some() {
						panic!("multiples values on functional property")
					}
					match object {
						Some(object) => {
							Some({
								match object { :: treeldr_rust_prelude :: rdf :: Object :: Id (id) => { :: treeldr_rust_prelude :: FromRdf :: from_rdf (namespace , id , graph) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: UnexpectedLiteralValue) }
							})
						}
						None => None,
					}
				},
				proof: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#proof"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				issuer: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#issuer"
							),
						)),
					);
					let object = objects.next();
					if objects.next().is_some() {
						panic!("multiples values on functional property")
					}
					match object {
						Some(object) => {
							Some({
								match object { :: treeldr_rust_prelude :: rdf :: Object :: Id (id) => { :: treeldr_rust_prelude :: FromRdf :: from_rdf (namespace , id , graph) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: UnexpectedLiteralValue) }
							})
						}
						None => None,
					}
				},
				type_: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::Id(id.clone())
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				id: { Some(::treeldr_rust_prelude::Id(id.clone())) },
			})
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
		for TermOfUse
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		fn from_rdf<G>(
			namespace: &mut N,
			id: &N::Id,
			graph: &G,
		) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
		where
			G: ::treeldr_rust_prelude::grdf::Graph<
				Subject = N::Id,
				Predicate = N::Id,
				Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
			>,
		{
			Ok(Self {})
		}
	}
	pub enum IssuerTriplesAndValues<'a, I, V> {
		Anyuri(
			::treeldr_rust_prelude::rdf::ValuesOnly<
				::treeldr_rust_prelude::rdf::LiteralValue<'a, ::iref::IriBuf, I, V>,
			>,
		),
		Object(super::rdfs::ResourceTriplesAndValues<'a, I, V>),
	}
	impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
		::treeldr_rust_prelude::RdfIterator<N> for IssuerTriplesAndValues<'a, N::Id, V>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::iref::IriBuf: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
	{
		type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
		fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&mut self,
			namespace: &mut N,
			generator: &mut G,
		) -> Option<Self::Item> {
			match self {
				Self::Anyuri(inner) => inner.next_with(namespace, generator),
				Self::Object(inner) => inner.next_with(namespace, generator),
			}
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
		::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for Issuer<N::Id>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::iref::IriBuf: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
	{
		type TriplesAndValues < 'a > = IssuerTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
		fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&'a self,
			namespace: &mut N,
			generator: &mut G,
		) -> Self::TriplesAndValues<'a>
		where
			N::Id: 'a,
			V: 'a,
		{
			match self {
				Self::Anyuri(value) => IssuerTriplesAndValues::Anyuri(
					value.unbound_rdf_triples_and_values(namespace, generator),
				),
				Self::Object(value) => IssuerTriplesAndValues::Object(
					value.unbound_rdf_triples_and_values(namespace, generator),
				),
			}
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N>
		for RefreshService
	where
		N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
	{
		fn into_json_ld(self, namespace: &N) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
			let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
			result.into()
		}
	}
	impl<C: ?Sized> super::rdfs::AnyResource<C> for TermOfUse {
		type Comment < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Comments < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Type < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Types < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Label < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Labels < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
			::std::iter::empty()
		}
		fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
			::std::iter::empty()
		}
		fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
			::std::iter::empty()
		}
	}
	impl<C: ?Sized + super::rdfs::AnyResourceProvider<I> + super::rdfs::ClassProvider<I>, I>
		super::vc::AnyVerifiableCredential<C> for Credential<I>
	{
		type CredentialSchema < 'a > = & 'a Schema where Self : 'a , C : 'a ;
		type CredentialSchemas < 'a > = :: std :: collections :: btree_set :: Iter < 'a , Schema > where Self : 'a , C : 'a ;
		type RefreshService < 'a > = & 'a RefreshService where Self : 'a , C : 'a ;
		type RefreshServices < 'a > = :: std :: collections :: btree_set :: Iter < 'a , RefreshService > where Self : 'a , C : 'a ;
		type Proof < 'a > = & 'a Proof where Self : 'a , C : 'a ;
		type Proofs < 'a > = :: std :: collections :: btree_set :: Iter < 'a , Proof > where Self : 'a , C : 'a ;
		type CredentialSubject < 'a > = & 'a super :: rdfs :: Resource < I > where Self : 'a , C : 'a ;
		type CredentialSubjects < 'a > = :: std :: collections :: btree_set :: Iter < 'a , super :: rdfs :: Resource < I > > where Self : 'a , C : 'a ;
		type IssuanceDate < 'a > = & 'a super :: xsd :: DateTime where Self : 'a , C : 'a ;
		type Issuer < 'a > = & 'a Issuer < I > where Self : 'a , C : 'a ;
		type Evidence < 'a > = & 'a Evidence where Self : 'a , C : 'a ;
		type Evidences < 'a > = :: std :: collections :: btree_set :: Iter < 'a , Evidence > where Self : 'a , C : 'a ;
		type CredentialStatus < 'a > = & 'a Status where Self : 'a , C : 'a ;
		type ExpirationDate < 'a > = & 'a super :: xsd :: DateTime where Self : 'a , C : 'a ;
		type TermsOfUse < 'a > = & 'a TermOfUse where Self : 'a , C : 'a ;
		type TermsOfUses < 'a > = :: std :: collections :: btree_set :: Iter < 'a , TermOfUse > where Self : 'a , C : 'a ;
		fn credential_schema<'a>(&'a self, context: &'a C) -> Self::CredentialSchemas<'a> {
			self.credential_schema.iter()
		}
		fn refresh_service<'a>(&'a self, context: &'a C) -> Self::RefreshServices<'a> {
			self.refresh_service.iter()
		}
		fn proof<'a>(&'a self, context: &'a C) -> Self::Proofs<'a> {
			self.proof.iter()
		}
		fn credential_subject<'a>(&'a self, context: &'a C) -> Self::CredentialSubjects<'a> {
			self.credential_subject.iter()
		}
		fn issuance_date<'a>(&'a self, context: &'a C) -> Self::IssuanceDate<'a> {
			&self.issuance_date
		}
		fn issuer<'a>(&'a self, context: &'a C) -> Option<Self::Issuer<'a>> {
			self.issuer.as_ref()
		}
		fn evidence<'a>(&'a self, context: &'a C) -> Self::Evidences<'a> {
			self.evidence.iter()
		}
		fn credential_status<'a>(&'a self, context: &'a C) -> Option<Self::CredentialStatus<'a>> {
			self.credential_status.as_ref()
		}
		fn expiration_date<'a>(&'a self, context: &'a C) -> Option<Self::ExpirationDate<'a>> {
			self.expiration_date.as_ref()
		}
		fn terms_of_use<'a>(&'a self, context: &'a C) -> Self::TermsOfUses<'a> {
			self.terms_of_use.iter()
		}
	}
	pub struct EvidenceTriplesAndValues<'a, I, V> {
		id_: Option<I>,
		_v: ::std::marker::PhantomData<&'a V>,
	}
	impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
		::treeldr_rust_prelude::RdfIterator<N> for EvidenceTriplesAndValues<'a, N::Id, V>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
		fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&mut self,
			vocabulary: &mut N,
			generator: &mut G,
		) -> Option<Self::Item> {
			self.id_
				.take()
				.map(::treeldr_rust_prelude::rdf_types::Object::Id)
				.map(::treeldr_rust_prelude::rdf::TripleOrValue::Value)
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
		::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for Evidence
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type TriplesAndValues < 'a > = EvidenceTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
		fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&'a self,
			namespace: &mut N,
			generator: &mut G,
		) -> Self::TriplesAndValues<'a>
		where
			N::Id: 'a,
			V: 'a,
		{
			EvidenceTriplesAndValues {
				id_: Some(generator.next(namespace)),
				_v: ::std::marker::PhantomData,
			}
		}
	}
}
pub mod org {
	pub trait AnyBlogPosting<C: ?Sized>: super::rdfs::AnyResource<C> {
		type Body<'a>: 'a + AnyText<C>
		where
			Self: 'a,
			C: 'a;
		type Title<'a>: 'a + AnyText<C>
		where
			Self: 'a,
			C: 'a;
		fn body<'a>(&'a self, context: &'a C) -> Option<Self::Body<'a>>;
		fn title<'a>(&'a self, context: &'a C) -> Option<Self::Title<'a>>;
	}
	pub trait AnyBlogPostingProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyBlogPosting>
	{
		type AnyBlogPosting: AnyBlogPosting<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyBlogPosting> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyBlogPosting>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyBlogPosting<C> for ::std::convert::Infallible {
		type Body < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Title < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		fn body<'a>(&'a self, _context: &'a C) -> Option<Self::Body<'a>> {
			unreachable!()
		}
		fn title<'a>(&'a self, _context: &'a C) -> Option<Self::Title<'a>> {
			unreachable!()
		}
	}
	impl<'r, C: ?Sized, T: AnyBlogPosting<C>> AnyBlogPosting<C> for &'r T {
		type Body < 'a > = T :: Body < 'a > where Self : 'a , C : 'a ;
		type Title < 'a > = T :: Title < 'a > where Self : 'a , C : 'a ;
		fn body<'a>(&'a self, context: &'a C) -> Option<Self::Body<'a>> {
			T::body(*self, context)
		}
		fn title<'a>(&'a self, context: &'a C) -> Option<Self::Title<'a>> {
			T::title(*self, context)
		}
	}
	pub trait AnyText<C: ?Sized>: super::rdfs::AnyResource<C> {}
	pub trait AnyTextProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyText>
	{
		type AnyText: AnyText<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyText> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyText>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyText<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: AnyText<C>> AnyText<C> for &'r T {}
	pub trait AnyThing<C: ?Sized>: super::rdfs::AnyResource<C> {
		type Name<'a>: 'a + AnyText<C>
		where
			Self: 'a,
			C: 'a;
		type Names<'a>: 'a + Iterator<Item = Self::Name<'a>>
		where
			Self: 'a,
			C: 'a;
		fn name<'a>(&'a self, context: &'a C) -> Self::Names<'a>;
	}
	pub trait AnyThingProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyThing>
	{
		type AnyThing: AnyThing<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyThing> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyThing>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyThing<C> for ::std::convert::Infallible {
		type Name < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Names < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn name<'a>(&'a self, _context: &'a C) -> Self::Names<'a> {
			unreachable!()
		}
	}
	impl<'r, C: ?Sized, T: AnyThing<C>> AnyThing<C> for &'r T {
		type Name < 'a > = T :: Name < 'a > where Self : 'a , C : 'a ;
		type Names < 'a > = T :: Names < 'a > where Self : 'a , C : 'a ;
		fn name<'a>(&'a self, context: &'a C) -> Self::Names<'a> {
			T::name(*self, context)
		}
	}
	pub trait AnyUrl<C: ?Sized>: super::rdfs::AnyResource<C> {}
	pub trait AnyUrlProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::AnyUrl> {
		type AnyUrl: AnyUrl<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyUrl> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyUrl>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyUrl<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: AnyUrl<C>> AnyUrl<C> for &'r T {}
	pub trait AnyPerson<C: ?Sized>: super::rdfs::AnyResource<C> {
		type Parent<'a>: 'a + AnyPerson<C>
		where
			Self: 'a,
			C: 'a;
		type Parents<'a>: 'a + Iterator<Item = Self::Parent<'a>>
		where
			Self: 'a,
			C: 'a;
		fn parent<'a>(&'a self, context: &'a C) -> Self::Parents<'a>;
	}
	pub trait AnyPersonProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyPerson>
	{
		type AnyPerson: AnyPerson<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyPerson> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyPerson>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyPerson<C> for ::std::convert::Infallible {
		type Parent < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Parents < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn parent<'a>(&'a self, _context: &'a C) -> Self::Parents<'a> {
			unreachable!()
		}
	}
	impl<'r, C: ?Sized, T: AnyPerson<C>> AnyPerson<C> for &'r T {
		type Parent < 'a > = T :: Parent < 'a > where Self : 'a , C : 'a ;
		type Parents < 'a > = T :: Parents < 'a > where Self : 'a , C : 'a ;
		fn parent<'a>(&'a self, context: &'a C) -> Self::Parents<'a> {
			T::parent(*self, context)
		}
	}
	#[doc = " Text."]
	pub type Text = ::std::string::String;
	#[doc = " Person."]
	#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
	pub struct Person {
		pub parent: std::collections::BTreeSet<Person>,
	}
	#[doc = " URL."]
	pub type Url = ::iref::IriBuf;
	#[doc = " Blog post."]
	#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
	pub struct BlogPosting {
		#[doc = " Content of the post."]
		pub body: Option<Text>,
		#[doc = " Title of the post."]
		pub title: Option<Text>,
	}
	#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
	pub struct Thing {
		pub name: std::collections::BTreeSet<Text>,
	}
	impl<C: ?Sized> AnyText<C> for ::std::string::String {}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N>
		for Person
	where
		N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
	{
		fn into_json_ld(self, namespace: &N) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
			let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
			if !self.parent.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("parent".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.parent
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			result.into()
		}
	}
	impl<C: ?Sized> AnyBlogPosting<C> for BlogPosting {
		type Body < 'a > = & 'a Text where Self : 'a , C : 'a ;
		type Title < 'a > = & 'a Text where Self : 'a , C : 'a ;
		fn body<'a>(&'a self, context: &'a C) -> Option<Self::Body<'a>> {
			self.body.as_ref()
		}
		fn title<'a>(&'a self, context: &'a C) -> Option<Self::Title<'a>> {
			self.title.as_ref()
		}
	}
	impl<C: ?Sized> super::rdfs::AnyResource<C> for BlogPosting {
		type Comment < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Comments < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Type < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Types < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Label < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Labels < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
			::std::iter::empty()
		}
		fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
			::std::iter::empty()
		}
		fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
			::std::iter::empty()
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
		for BlogPosting
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::std::string::String: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>,
	{
		fn from_rdf<G>(
			namespace: &mut N,
			id: &N::Id,
			graph: &G,
		) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
		where
			G: ::treeldr_rust_prelude::grdf::Graph<
				Subject = N::Id,
				Predicate = N::Id,
				Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
			>,
		{
			Ok(Self {
				body: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!("https://schema.org/body"),
						)),
					);
					let object = objects.next();
					if objects.next().is_some() {
						panic!("multiples values on functional property")
					}
					match object {
						Some(object) => Some({
							match object { :: treeldr_rust_prelude :: rdf :: Object :: Literal (lit) => { < String as :: treeldr_rust_prelude :: rdf :: FromLiteral < V , N >> :: from_literal (namespace , lit) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: ExpectedLiteralValue) }
						}),
						None => None,
					}
				},
				title: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!("https://schema.org/title"),
						)),
					);
					let object = objects.next();
					if objects.next().is_some() {
						panic!("multiples values on functional property")
					}
					match object {
						Some(object) => Some({
							match object { :: treeldr_rust_prelude :: rdf :: Object :: Literal (lit) => { < String as :: treeldr_rust_prelude :: rdf :: FromLiteral < V , N >> :: from_literal (namespace , lit) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: ExpectedLiteralValue) }
						}),
						None => None,
					}
				},
			})
		}
	}
	impl<C: ?Sized> AnyUrl<C> for ::iref::IriBuf {}
	pub struct PersonTriplesAndValues<'a, I, V> {
		id_: Option<I>,
		parent: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, Person>,
			PersonTriplesAndValues<'a, I, V>,
			V,
		>,
	}
	impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
		::treeldr_rust_prelude::RdfIterator<N> for PersonTriplesAndValues<'a, N::Id, V>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
		fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&mut self,
			vocabulary: &mut N,
			generator: &mut G,
		) -> Option<Self::Item> {
			self.parent
				.next_with(vocabulary, generator)
				.map(|item| match item {
					::treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple) => {
						treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple)
					}
					treeldr_rust_prelude::rdf::TripleOrValue::Value(value) => {
						treeldr_rust_prelude::rdf::TripleOrValue::Triple(::rdf_types::Triple(
							self.id_.clone().unwrap(),
							treeldr_rust_prelude::rdf_types::FromIri::from_iri(vocabulary.insert(
								::treeldr_rust_prelude::static_iref::iri!(
									"https://schema.org/parent"
								),
							)),
							value,
						))
					}
				})
				.or_else(|| {
					self.id_
						.take()
						.map(::treeldr_rust_prelude::rdf_types::Object::Id)
						.map(::treeldr_rust_prelude::rdf::TripleOrValue::Value)
				})
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
		::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for Person
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type TriplesAndValues < 'a > = PersonTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
		fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&'a self,
			namespace: &mut N,
			generator: &mut G,
		) -> Self::TriplesAndValues<'a>
		where
			N::Id: 'a,
			V: 'a,
		{
			PersonTriplesAndValues {
				id_: Some(generator.next(namespace)),
				parent: self
					.parent
					.unbound_rdf_triples_and_values(namespace, generator),
			}
		}
	}
	impl<C: ?Sized> super::rdfs::AnyResource<C> for Person {
		type Comment < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Comments < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Type < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Types < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Label < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Labels < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
			::std::iter::empty()
		}
		fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
			::std::iter::empty()
		}
		fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
			::std::iter::empty()
		}
	}
	pub struct BlogPostingTriplesAndValues<'a, I, V> {
		id_: Option<I>,
		body: ::treeldr_rust_prelude::rdf::iter::Optional<
			::treeldr_rust_prelude::rdf::ValuesOnly<
				::treeldr_rust_prelude::rdf::LiteralValue<'a, ::std::string::String, I, V>,
			>,
		>,
		title: ::treeldr_rust_prelude::rdf::iter::Optional<
			::treeldr_rust_prelude::rdf::ValuesOnly<
				::treeldr_rust_prelude::rdf::LiteralValue<'a, ::std::string::String, I, V>,
			>,
		>,
	}
	impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
		::treeldr_rust_prelude::RdfIterator<N> for BlogPostingTriplesAndValues<'a, N::Id, V>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
	{
		type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
		fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&mut self,
			vocabulary: &mut N,
			generator: &mut G,
		) -> Option<Self::Item> {
			self.title
				.next_with(vocabulary, generator)
				.map(|item| match item {
					::treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple) => {
						treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple)
					}
					treeldr_rust_prelude::rdf::TripleOrValue::Value(value) => {
						treeldr_rust_prelude::rdf::TripleOrValue::Triple(::rdf_types::Triple(
							self.id_.clone().unwrap(),
							treeldr_rust_prelude::rdf_types::FromIri::from_iri(vocabulary.insert(
								::treeldr_rust_prelude::static_iref::iri!(
									"https://schema.org/title"
								),
							)),
							value,
						))
					}
				})
				.or_else(|| {
					self.body
						.next_with(vocabulary, generator)
						.map(|item| match item {
							::treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple) => {
								treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple)
							}
							treeldr_rust_prelude::rdf::TripleOrValue::Value(value) => {
								treeldr_rust_prelude::rdf::TripleOrValue::Triple(
									::rdf_types::Triple(
										self.id_.clone().unwrap(),
										treeldr_rust_prelude::rdf_types::FromIri::from_iri(
											vocabulary.insert(
												::treeldr_rust_prelude::static_iref::iri!(
													"https://schema.org/body"
												),
											),
										),
										value,
									),
								)
							}
						})
						.or_else(|| {
							self.id_
								.take()
								.map(::treeldr_rust_prelude::rdf_types::Object::Id)
								.map(::treeldr_rust_prelude::rdf::TripleOrValue::Value)
						})
				})
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
		::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for BlogPosting
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
	{
		type TriplesAndValues < 'a > = BlogPostingTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
		fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&'a self,
			namespace: &mut N,
			generator: &mut G,
		) -> Self::TriplesAndValues<'a>
		where
			N::Id: 'a,
			V: 'a,
		{
			BlogPostingTriplesAndValues {
				id_: Some(generator.next(namespace)),
				body: self
					.body
					.unbound_rdf_triples_and_values(namespace, generator),
				title: self
					.title
					.unbound_rdf_triples_and_values(namespace, generator),
			}
		}
	}
	impl<C: ?Sized> AnyPerson<C> for Person {
		type Parent < 'a > = & 'a Person where Self : 'a , C : 'a ;
		type Parents < 'a > = :: std :: collections :: btree_set :: Iter < 'a , Person > where Self : 'a , C : 'a ;
		fn parent<'a>(&'a self, context: &'a C) -> Self::Parents<'a> {
			self.parent.iter()
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
		for Thing
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::std::string::String: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>,
	{
		fn from_rdf<G>(
			namespace: &mut N,
			id: &N::Id,
			graph: &G,
		) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
		where
			G: ::treeldr_rust_prelude::grdf::Graph<
				Subject = N::Id,
				Predicate = N::Id,
				Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
			>,
		{
			Ok(Self {
				name: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!("https://schema.org/name"),
						)),
					) {
						result . insert (match object { :: treeldr_rust_prelude :: rdf :: Object :: Literal (lit) => { < String as :: treeldr_rust_prelude :: rdf :: FromLiteral < V , N >> :: from_literal (namespace , lit) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: ExpectedLiteralValue) }) ;
					}
					result
				},
			})
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N>
		for Thing
	where
		N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
	{
		fn into_json_ld(self, namespace: &N) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
			let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
			if !self.name.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("name".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.name
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			result.into()
		}
	}
	pub struct ThingTriplesAndValues<'a, I, V> {
		id_: Option<I>,
		name: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, Text>,
			::treeldr_rust_prelude::rdf::ValuesOnly<
				::treeldr_rust_prelude::rdf::LiteralValue<'a, ::std::string::String, I, V>,
			>,
			V,
		>,
	}
	impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
		::treeldr_rust_prelude::RdfIterator<N> for ThingTriplesAndValues<'a, N::Id, V>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
	{
		type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
		fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&mut self,
			vocabulary: &mut N,
			generator: &mut G,
		) -> Option<Self::Item> {
			self.name
				.next_with(vocabulary, generator)
				.map(|item| match item {
					::treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple) => {
						treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple)
					}
					treeldr_rust_prelude::rdf::TripleOrValue::Value(value) => {
						treeldr_rust_prelude::rdf::TripleOrValue::Triple(::rdf_types::Triple(
							self.id_.clone().unwrap(),
							treeldr_rust_prelude::rdf_types::FromIri::from_iri(vocabulary.insert(
								::treeldr_rust_prelude::static_iref::iri!(
									"https://schema.org/name"
								),
							)),
							value,
						))
					}
				})
				.or_else(|| {
					self.id_
						.take()
						.map(::treeldr_rust_prelude::rdf_types::Object::Id)
						.map(::treeldr_rust_prelude::rdf::TripleOrValue::Value)
				})
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
		::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for Thing
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
	{
		type TriplesAndValues < 'a > = ThingTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
		fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&'a self,
			namespace: &mut N,
			generator: &mut G,
		) -> Self::TriplesAndValues<'a>
		where
			N::Id: 'a,
			V: 'a,
		{
			ThingTriplesAndValues {
				id_: Some(generator.next(namespace)),
				name: self
					.name
					.unbound_rdf_triples_and_values(namespace, generator),
			}
		}
	}
	impl<C: ?Sized> super::rdfs::AnyResource<C> for Thing {
		type Comment < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Comments < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Type < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Types < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Label < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Labels < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
			::std::iter::empty()
		}
		fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
			::std::iter::empty()
		}
		fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
			::std::iter::empty()
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N>
		for BlogPosting
	where
		N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
	{
		fn into_json_ld(self, namespace: &N) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
			let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
			if let Some(value) = self.body {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("body".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			if let Some(value) = self.title {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("title".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			result.into()
		}
	}
	impl<C: ?Sized> AnyThing<C> for Thing {
		type Name < 'a > = & 'a Text where Self : 'a , C : 'a ;
		type Names < 'a > = :: std :: collections :: btree_set :: Iter < 'a , Text > where Self : 'a , C : 'a ;
		fn name<'a>(&'a self, context: &'a C) -> Self::Names<'a> {
			self.name.iter()
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
		for Person
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		fn from_rdf<G>(
			namespace: &mut N,
			id: &N::Id,
			graph: &G,
		) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
		where
			G: ::treeldr_rust_prelude::grdf::Graph<
				Subject = N::Id,
				Predicate = N::Id,
				Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
			>,
		{
			Ok(Self {
				parent: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!("https://schema.org/parent"),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
			})
		}
	}
}
pub mod basic_post {
	pub trait AnyVerifiableBasicPost<C: ?Sized> {}
	pub trait AnyVerifiableBasicPostProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyVerifiableBasicPost>
	{
		type AnyVerifiableBasicPost: AnyVerifiableBasicPost<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyVerifiableBasicPost> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyVerifiableBasicPost>>::get(
				self, id,
			)
		}
	}
	impl<C: ?Sized> AnyVerifiableBasicPost<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: AnyVerifiableBasicPost<C>> AnyVerifiableBasicPost<C> for &'r T {}
	#[doc = " Verifiable `BasicPost`."]
	#[doc = ""]
	#[doc = "Defined as a `vc:VerifiableCredential` where the"]
	#[doc = " credential subject is a `schema:BlogPosting`."]
	#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
	pub struct VerifiableBasicPost<I> {
		pub expiration_date: Option<super::xsd::DateTime>,
		pub issuance_date: super::xsd::DateTime,
		pub refresh_service: std::collections::BTreeSet<super::rebase::RefreshService>,
		pub credential_schema: std::collections::BTreeSet<super::rebase::Schema>,
		pub evidence: std::collections::BTreeSet<super::rebase::Evidence>,
		pub terms_of_use: std::collections::BTreeSet<super::rebase::TermOfUse>,
		pub credential_status: Option<super::rebase::Status>,
		pub proof: std::collections::BTreeSet<super::rebase::Proof>,
		pub issuer: Option<super::rebase::Issuer<I>>,
		#[doc = " Type of the credential."]
		pub type_: std::collections::BTreeSet<::treeldr_rust_prelude::Id<I>>,
		#[doc = " Identifier of the credential."]
		pub id: Option<::treeldr_rust_prelude::Id<I>>,
		pub credential_subject: std::collections::BTreeSet<BasicPost<I>>,
	}
	impl<I> VerifiableBasicPost<I> {
		pub fn new(issuance_date: super::xsd::DateTime) -> Self {
			Self {
				expiration_date: Default::default(),
				issuance_date: issuance_date,
				refresh_service: Default::default(),
				credential_schema: Default::default(),
				evidence: Default::default(),
				terms_of_use: Default::default(),
				credential_status: Default::default(),
				proof: Default::default(),
				issuer: Default::default(),
				type_: Default::default(),
				id: Default::default(),
				credential_subject: Default::default(),
			}
		}
	}
	#[doc = " Basic Blog Post."]
	#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
	pub struct BasicPost<I> {
		#[doc = " Content of the post."]
		pub body: Option<super::org::Text>,
		#[doc = " Title of the post."]
		pub title: Option<super::org::Text>,
		#[doc = " Identifier of the post."]
		pub id: ::treeldr_rust_prelude::Id<I>,
	}
	impl<I> BasicPost<I> {
		pub fn new(id: ::treeldr_rust_prelude::Id<I>) -> Self {
			Self {
				body: Default::default(),
				title: Default::default(),
				id: id,
			}
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N>
		for VerifiableBasicPost<N::Id>
	where
		N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
	{
		fn into_json_ld(self, namespace: &N) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
			let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
			if let Some(value) = self.expiration_date {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("expirationDate".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			result.insert(
				::treeldr_rust_prelude::locspan::Meta("issuanceDate".into(), ()),
				::treeldr_rust_prelude::locspan::Meta(
					::treeldr_rust_prelude::IntoJsonLd::into_json_ld(self.issuance_date, namespace),
					(),
				),
			);
			if !self.refresh_service.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("refreshService".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.refresh_service
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if !self.credential_schema.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("credentialSchema".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.credential_schema
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if !self.evidence.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("evidence".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.evidence
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if !self.terms_of_use.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("termsOfUse".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.terms_of_use
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if let Some(value) = self.credential_status {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("credentialStatus".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			if !self.proof.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("proof".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.proof
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if let Some(value) = self.issuer {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("issuer".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			if !self.type_.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("type".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.type_
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if let Some(value) = self.id {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("id".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			if !self.credential_subject.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("credentialSubject".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.credential_subject
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			result.into()
		}
	}
	impl<C: ?Sized + super::rdfs::AnyResourceProvider<I> + super::rdfs::ClassProvider<I>, I>
		AnyVerifiableBasicPost<C> for VerifiableBasicPost<I>
	{
	}
	pub struct BasicPostTriplesAndValues<'a, I, V> {
		id_: Option<I>,
		body: ::treeldr_rust_prelude::rdf::iter::Optional<
			::treeldr_rust_prelude::rdf::ValuesOnly<
				::treeldr_rust_prelude::rdf::LiteralValue<'a, ::std::string::String, I, V>,
			>,
		>,
		title: ::treeldr_rust_prelude::rdf::iter::Optional<
			::treeldr_rust_prelude::rdf::ValuesOnly<
				::treeldr_rust_prelude::rdf::LiteralValue<'a, ::std::string::String, I, V>,
			>,
		>,
	}
	impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
		::treeldr_rust_prelude::RdfIterator<N> for BasicPostTriplesAndValues<'a, N::Id, V>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
	{
		type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
		fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&mut self,
			vocabulary: &mut N,
			generator: &mut G,
		) -> Option<Self::Item> {
			self.title
				.next_with(vocabulary, generator)
				.map(|item| match item {
					::treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple) => {
						treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple)
					}
					treeldr_rust_prelude::rdf::TripleOrValue::Value(value) => {
						treeldr_rust_prelude::rdf::TripleOrValue::Triple(::rdf_types::Triple(
							self.id_.clone().unwrap(),
							treeldr_rust_prelude::rdf_types::FromIri::from_iri(vocabulary.insert(
								::treeldr_rust_prelude::static_iref::iri!(
									"https://schema.org/title"
								),
							)),
							value,
						))
					}
				})
				.or_else(|| {
					self.body
						.next_with(vocabulary, generator)
						.map(|item| match item {
							::treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple) => {
								treeldr_rust_prelude::rdf::TripleOrValue::Triple(triple)
							}
							treeldr_rust_prelude::rdf::TripleOrValue::Value(value) => {
								treeldr_rust_prelude::rdf::TripleOrValue::Triple(
									::rdf_types::Triple(
										self.id_.clone().unwrap(),
										treeldr_rust_prelude::rdf_types::FromIri::from_iri(
											vocabulary.insert(
												::treeldr_rust_prelude::static_iref::iri!(
													"https://schema.org/body"
												),
											),
										),
										value,
									),
								)
							}
						})
						.or_else(|| {
							self.id_
								.take()
								.map(::treeldr_rust_prelude::rdf_types::Object::Id)
								.map(::treeldr_rust_prelude::rdf::TripleOrValue::Value)
						})
				})
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
		::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for BasicPost<N::Id>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
	{
		type TriplesAndValues < 'a > = BasicPostTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
		fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&'a self,
			namespace: &mut N,
			generator: &mut G,
		) -> Self::TriplesAndValues<'a>
		where
			N::Id: 'a,
			V: 'a,
		{
			BasicPostTriplesAndValues {
				id_: Some(self.id.clone().unwrap()),
				body: self
					.body
					.unbound_rdf_triples_and_values(namespace, generator),
				title: self
					.title
					.unbound_rdf_triples_and_values(namespace, generator),
			}
		}
	}
	pub struct VerifiableBasicPostTriplesAndValues<'a, I, V> {
		id_: Option<I>,
		expiration_date: ::treeldr_rust_prelude::rdf::iter::Optional<
			::treeldr_rust_prelude::rdf::ValuesOnly<
				::treeldr_rust_prelude::rdf::LiteralValue<
					'a,
					::chrono::DateTime<::chrono::Utc>,
					I,
					V,
				>,
			>,
		>,
		issuance_date: ::treeldr_rust_prelude::rdf::ValuesOnly<
			::treeldr_rust_prelude::rdf::LiteralValue<'a, ::chrono::DateTime<::chrono::Utc>, I, V>,
		>,
		refresh_service: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, super::rebase::RefreshService>,
			super::rebase::RefreshServiceTriplesAndValues<'a, I, V>,
			V,
		>,
		credential_schema: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, super::rebase::Schema>,
			super::rebase::SchemaTriplesAndValues<'a, I, V>,
			V,
		>,
		evidence: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, super::rebase::Evidence>,
			super::rebase::EvidenceTriplesAndValues<'a, I, V>,
			V,
		>,
		terms_of_use: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, super::rebase::TermOfUse>,
			super::rebase::TermOfUseTriplesAndValues<'a, I, V>,
			V,
		>,
		credential_status: ::treeldr_rust_prelude::rdf::iter::Optional<
			super::rebase::StatusTriplesAndValues<'a, I, V>,
		>,
		proof: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, super::rebase::Proof>,
			super::rebase::ProofTriplesAndValues<'a, I, V>,
			V,
		>,
		issuer: ::treeldr_rust_prelude::rdf::iter::Optional<
			super::rebase::IssuerTriplesAndValues<'a, I, V>,
		>,
		type_: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, ::treeldr_rust_prelude::Id<I>>,
			::treeldr_rust_prelude::rdf::ValuesOnly<::treeldr_rust_prelude::rdf::IdValue<'a, I, V>>,
			V,
		>,
		credential_subject: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, BasicPost<I>>,
			BasicPostTriplesAndValues<'a, I, V>,
			V,
		>,
	}
	impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
		::treeldr_rust_prelude::RdfIterator<N> for VerifiableBasicPostTriplesAndValues<'a, N::Id, V>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
		::chrono::DateTime<::chrono::Utc>: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
		::iref::IriBuf: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
	{
		type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
		fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&mut self,
			vocabulary: &mut N,
			generator: &mut G,
		) -> Option<Self::Item> {
			self . credential_subject . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#credentialSubject"))) , value)) } }) . or_else (|| self . type_ . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"))) , value)) } }) . or_else (|| self . issuer . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#issuer"))) , value)) } }) . or_else (|| self . proof . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#proof"))) , value)) } }) . or_else (|| self . credential_status . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#credentialStatus"))) , value)) } }) . or_else (|| self . terms_of_use . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#termsOfUse"))) , value)) } }) . or_else (|| self . evidence . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#evidence"))) , value)) } }) . or_else (|| self . credential_schema . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#credentialSchema"))) , value)) } }) . or_else (|| self . refresh_service . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#refreshService"))) , value)) } }) . or_else (|| self . issuance_date . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#issuanceDate"))) , value)) } }) . or_else (|| self . expiration_date . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#expirationDate"))) , value)) } }) . or_else (|| self . id_ . take () . map (:: treeldr_rust_prelude :: rdf_types :: Object :: Id) . map (:: treeldr_rust_prelude :: rdf :: TripleOrValue :: Value))))))))))))
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
		::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for VerifiableBasicPost<N::Id>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
		::chrono::DateTime<::chrono::Utc>: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
		::iref::IriBuf: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
	{
		type TriplesAndValues < 'a > = VerifiableBasicPostTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
		fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&'a self,
			namespace: &mut N,
			generator: &mut G,
		) -> Self::TriplesAndValues<'a>
		where
			N::Id: 'a,
			V: 'a,
		{
			VerifiableBasicPostTriplesAndValues {
				id_: Some(
					self.id
						.clone()
						.map(::treeldr_rust_prelude::Id::unwrap)
						.unwrap_or_else(|| generator.next(namespace)),
				),
				expiration_date: self
					.expiration_date
					.unbound_rdf_triples_and_values(namespace, generator),
				issuance_date: self
					.issuance_date
					.unbound_rdf_triples_and_values(namespace, generator),
				refresh_service: self
					.refresh_service
					.unbound_rdf_triples_and_values(namespace, generator),
				credential_schema: self
					.credential_schema
					.unbound_rdf_triples_and_values(namespace, generator),
				evidence: self
					.evidence
					.unbound_rdf_triples_and_values(namespace, generator),
				terms_of_use: self
					.terms_of_use
					.unbound_rdf_triples_and_values(namespace, generator),
				credential_status: self
					.credential_status
					.unbound_rdf_triples_and_values(namespace, generator),
				proof: self
					.proof
					.unbound_rdf_triples_and_values(namespace, generator),
				issuer: self
					.issuer
					.unbound_rdf_triples_and_values(namespace, generator),
				type_: self
					.type_
					.unbound_rdf_triples_and_values(namespace, generator),
				credential_subject: self
					.credential_subject
					.unbound_rdf_triples_and_values(namespace, generator),
			}
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N>
		for BasicPost<N::Id>
	where
		N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
	{
		fn into_json_ld(self, namespace: &N) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
			let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
			if let Some(value) = self.body {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("body".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			if let Some(value) = self.title {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("title".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			result.insert(
				::treeldr_rust_prelude::locspan::Meta("id".into(), ()),
				::treeldr_rust_prelude::locspan::Meta(
					::treeldr_rust_prelude::IntoJsonLd::into_json_ld(self.id, namespace),
					(),
				),
			);
			result.into()
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
		for VerifiableBasicPost<N::Id>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::std::string::String: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>,
		::chrono::DateTime<::chrono::Utc>: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>,
		::iref::IriBuf: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>,
	{
		fn from_rdf<G>(
			namespace: &mut N,
			id: &N::Id,
			graph: &G,
		) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
		where
			G: ::treeldr_rust_prelude::grdf::Graph<
				Subject = N::Id,
				Predicate = N::Id,
				Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
			>,
		{
			Ok(Self {
				expiration_date: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#expirationDate"
							),
						)),
					);
					let object = objects.next();
					if objects.next().is_some() {
						panic!("multiples values on functional property")
					}
					match object {
						Some(object) => Some({
							match object { :: treeldr_rust_prelude :: rdf :: Object :: Literal (lit) => { < :: chrono :: DateTime < :: chrono :: Utc > as :: treeldr_rust_prelude :: rdf :: FromLiteral < V , N >> :: from_literal (namespace , lit) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: ExpectedLiteralValue) }
						}),
						None => None,
					}
				},
				issuance_date: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#issuanceDate"
							),
						)),
					);
					match objects.next() {
						Some(object) => {
							if objects.next().is_some() {
								panic!("multiples values on functional property")
							}
							match object { :: treeldr_rust_prelude :: rdf :: Object :: Literal (lit) => { < :: chrono :: DateTime < :: chrono :: Utc > as :: treeldr_rust_prelude :: rdf :: FromLiteral < V , N >> :: from_literal (namespace , lit) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: ExpectedLiteralValue) }
						}
						None => {
							return Err(
								::treeldr_rust_prelude::FromRdfError::MissingRequiredPropertyValue,
							)
						}
					}
				},
				refresh_service: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#refreshService"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				credential_schema: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#credentialSchema"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				evidence: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#evidence"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				terms_of_use: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#termsOfUse"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				credential_status: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#credentialStatus"
							),
						)),
					);
					let object = objects.next();
					if objects.next().is_some() {
						panic!("multiples values on functional property")
					}
					match object {
						Some(object) => {
							Some({
								match object { :: treeldr_rust_prelude :: rdf :: Object :: Id (id) => { :: treeldr_rust_prelude :: FromRdf :: from_rdf (namespace , id , graph) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: UnexpectedLiteralValue) }
							})
						}
						None => None,
					}
				},
				proof: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#proof"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				issuer: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#issuer"
							),
						)),
					);
					let object = objects.next();
					if objects.next().is_some() {
						panic!("multiples values on functional property")
					}
					match object {
						Some(object) => {
							Some({
								match object { :: treeldr_rust_prelude :: rdf :: Object :: Id (id) => { :: treeldr_rust_prelude :: FromRdf :: from_rdf (namespace , id , graph) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: UnexpectedLiteralValue) }
							})
						}
						None => None,
					}
				},
				type_: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::Id(id.clone())
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				id: { Some(::treeldr_rust_prelude::Id(id.clone())) },
				credential_subject: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#credentialSubject"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
			})
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
		for BasicPost<N::Id>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::std::string::String: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>,
	{
		fn from_rdf<G>(
			namespace: &mut N,
			id: &N::Id,
			graph: &G,
		) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
		where
			G: ::treeldr_rust_prelude::grdf::Graph<
				Subject = N::Id,
				Predicate = N::Id,
				Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
			>,
		{
			Ok(Self {
				body: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!("https://schema.org/body"),
						)),
					);
					let object = objects.next();
					if objects.next().is_some() {
						panic!("multiples values on functional property")
					}
					match object {
						Some(object) => Some({
							match object { :: treeldr_rust_prelude :: rdf :: Object :: Literal (lit) => { < String as :: treeldr_rust_prelude :: rdf :: FromLiteral < V , N >> :: from_literal (namespace , lit) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: ExpectedLiteralValue) }
						}),
						None => None,
					}
				},
				title: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!("https://schema.org/title"),
						)),
					);
					let object = objects.next();
					if objects.next().is_some() {
						panic!("multiples values on functional property")
					}
					match object {
						Some(object) => Some({
							match object { :: treeldr_rust_prelude :: rdf :: Object :: Literal (lit) => { < String as :: treeldr_rust_prelude :: rdf :: FromLiteral < V , N >> :: from_literal (namespace , lit) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: ExpectedLiteralValue) }
						}),
						None => None,
					}
				},
				id: { ::treeldr_rust_prelude::Id(id.clone()) },
			})
		}
	}
	impl<C: ?Sized + super::rdfs::AnyResourceProvider<I>, I> super::rdfs::AnyResource<C>
		for BasicPost<I>
	{
		type Comment < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Comments < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Type < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Types < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Label < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Labels < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
			::std::iter::empty()
		}
		fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
			::std::iter::empty()
		}
		fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
			::std::iter::empty()
		}
	}
	impl<C: ?Sized + super::rdfs::AnyResourceProvider<I>, I> super::org::AnyBlogPosting<C>
		for BasicPost<I>
	{
		type Body < 'a > = & 'a super :: org :: Text where Self : 'a , C : 'a ;
		type Title < 'a > = & 'a super :: org :: Text where Self : 'a , C : 'a ;
		fn body<'a>(&'a self, context: &'a C) -> Option<Self::Body<'a>> {
			self.body.as_ref()
		}
		fn title<'a>(&'a self, context: &'a C) -> Option<Self::Title<'a>> {
			self.title.as_ref()
		}
	}
}
pub mod vc {
	pub trait AnyVerifiableCredential<C: ?Sized>: super::rdfs::AnyResource<C> {
		type CredentialSchema<'a>: 'a + super::rdfs::AnyResource<C>
		where
			Self: 'a,
			C: 'a;
		type CredentialSchemas<'a>: 'a + Iterator<Item = Self::CredentialSchema<'a>>
		where
			Self: 'a,
			C: 'a;
		type RefreshService<'a>: 'a + super::rdfs::AnyResource<C>
		where
			Self: 'a,
			C: 'a;
		type RefreshServices<'a>: 'a + Iterator<Item = Self::RefreshService<'a>>
		where
			Self: 'a,
			C: 'a;
		type Proof<'a>: 'a + super::rdfs::AnyResource<C>
		where
			Self: 'a,
			C: 'a;
		type Proofs<'a>: 'a + Iterator<Item = Self::Proof<'a>>
		where
			Self: 'a,
			C: 'a;
		type CredentialSubject<'a>: 'a + super::rdfs::AnyResource<C>
		where
			Self: 'a,
			C: 'a;
		type CredentialSubjects<'a>: 'a + Iterator<Item = Self::CredentialSubject<'a>>
		where
			Self: 'a,
			C: 'a;
		type IssuanceDate<'a>: 'a + super::xsd::AnyDateTime<C>
		where
			Self: 'a,
			C: 'a;
		type Issuer<'a>: 'a + super::rdfs::AnyResource<C>
		where
			Self: 'a,
			C: 'a;
		type Evidence<'a>: 'a + super::rdfs::AnyResource<C>
		where
			Self: 'a,
			C: 'a;
		type Evidences<'a>: 'a + Iterator<Item = Self::Evidence<'a>>
		where
			Self: 'a,
			C: 'a;
		type CredentialStatus<'a>: 'a + super::rdfs::AnyResource<C>
		where
			Self: 'a,
			C: 'a;
		type ExpirationDate<'a>: 'a + super::xsd::AnyDateTime<C>
		where
			Self: 'a,
			C: 'a;
		type TermsOfUse<'a>: 'a + super::rdfs::AnyResource<C>
		where
			Self: 'a,
			C: 'a;
		type TermsOfUses<'a>: 'a + Iterator<Item = Self::TermsOfUse<'a>>
		where
			Self: 'a,
			C: 'a;
		fn credential_schema<'a>(&'a self, context: &'a C) -> Self::CredentialSchemas<'a>;
		fn refresh_service<'a>(&'a self, context: &'a C) -> Self::RefreshServices<'a>;
		fn proof<'a>(&'a self, context: &'a C) -> Self::Proofs<'a>;
		fn credential_subject<'a>(&'a self, context: &'a C) -> Self::CredentialSubjects<'a>;
		fn issuance_date<'a>(&'a self, context: &'a C) -> Self::IssuanceDate<'a>;
		fn issuer<'a>(&'a self, context: &'a C) -> Option<Self::Issuer<'a>>;
		fn evidence<'a>(&'a self, context: &'a C) -> Self::Evidences<'a>;
		fn credential_status<'a>(&'a self, context: &'a C) -> Option<Self::CredentialStatus<'a>>;
		fn expiration_date<'a>(&'a self, context: &'a C) -> Option<Self::ExpirationDate<'a>>;
		fn terms_of_use<'a>(&'a self, context: &'a C) -> Self::TermsOfUses<'a>;
	}
	pub trait AnyVerifiableCredentialProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyVerifiableCredential>
	{
		type AnyVerifiableCredential: AnyVerifiableCredential<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyVerifiableCredential> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyVerifiableCredential>>::get(
				self, id,
			)
		}
	}
	impl<C: ?Sized> AnyVerifiableCredential<C> for ::std::convert::Infallible {
		type CredentialSchema < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type CredentialSchemas < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type RefreshService < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type RefreshServices < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Proof < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Proofs < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type CredentialSubject < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type CredentialSubjects < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type IssuanceDate < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Issuer < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Evidence < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Evidences < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type CredentialStatus < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type ExpirationDate < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type TermsOfUse < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type TermsOfUses < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn credential_schema<'a>(&'a self, _context: &'a C) -> Self::CredentialSchemas<'a> {
			unreachable!()
		}
		fn refresh_service<'a>(&'a self, _context: &'a C) -> Self::RefreshServices<'a> {
			unreachable!()
		}
		fn proof<'a>(&'a self, _context: &'a C) -> Self::Proofs<'a> {
			unreachable!()
		}
		fn credential_subject<'a>(&'a self, _context: &'a C) -> Self::CredentialSubjects<'a> {
			unreachable!()
		}
		fn issuance_date<'a>(&'a self, _context: &'a C) -> Self::IssuanceDate<'a> {
			unreachable!()
		}
		fn issuer<'a>(&'a self, _context: &'a C) -> Option<Self::Issuer<'a>> {
			unreachable!()
		}
		fn evidence<'a>(&'a self, _context: &'a C) -> Self::Evidences<'a> {
			unreachable!()
		}
		fn credential_status<'a>(&'a self, _context: &'a C) -> Option<Self::CredentialStatus<'a>> {
			unreachable!()
		}
		fn expiration_date<'a>(&'a self, _context: &'a C) -> Option<Self::ExpirationDate<'a>> {
			unreachable!()
		}
		fn terms_of_use<'a>(&'a self, _context: &'a C) -> Self::TermsOfUses<'a> {
			unreachable!()
		}
	}
	impl<'r, C: ?Sized, T: AnyVerifiableCredential<C>> AnyVerifiableCredential<C> for &'r T {
		type CredentialSchema < 'a > = T :: CredentialSchema < 'a > where Self : 'a , C : 'a ;
		type CredentialSchemas < 'a > = T :: CredentialSchemas < 'a > where Self : 'a , C : 'a ;
		type RefreshService < 'a > = T :: RefreshService < 'a > where Self : 'a , C : 'a ;
		type RefreshServices < 'a > = T :: RefreshServices < 'a > where Self : 'a , C : 'a ;
		type Proof < 'a > = T :: Proof < 'a > where Self : 'a , C : 'a ;
		type Proofs < 'a > = T :: Proofs < 'a > where Self : 'a , C : 'a ;
		type CredentialSubject < 'a > = T :: CredentialSubject < 'a > where Self : 'a , C : 'a ;
		type CredentialSubjects < 'a > = T :: CredentialSubjects < 'a > where Self : 'a , C : 'a ;
		type IssuanceDate < 'a > = T :: IssuanceDate < 'a > where Self : 'a , C : 'a ;
		type Issuer < 'a > = T :: Issuer < 'a > where Self : 'a , C : 'a ;
		type Evidence < 'a > = T :: Evidence < 'a > where Self : 'a , C : 'a ;
		type Evidences < 'a > = T :: Evidences < 'a > where Self : 'a , C : 'a ;
		type CredentialStatus < 'a > = T :: CredentialStatus < 'a > where Self : 'a , C : 'a ;
		type ExpirationDate < 'a > = T :: ExpirationDate < 'a > where Self : 'a , C : 'a ;
		type TermsOfUse < 'a > = T :: TermsOfUse < 'a > where Self : 'a , C : 'a ;
		type TermsOfUses < 'a > = T :: TermsOfUses < 'a > where Self : 'a , C : 'a ;
		fn credential_schema<'a>(&'a self, context: &'a C) -> Self::CredentialSchemas<'a> {
			T::credential_schema(*self, context)
		}
		fn refresh_service<'a>(&'a self, context: &'a C) -> Self::RefreshServices<'a> {
			T::refresh_service(*self, context)
		}
		fn proof<'a>(&'a self, context: &'a C) -> Self::Proofs<'a> {
			T::proof(*self, context)
		}
		fn credential_subject<'a>(&'a self, context: &'a C) -> Self::CredentialSubjects<'a> {
			T::credential_subject(*self, context)
		}
		fn issuance_date<'a>(&'a self, context: &'a C) -> Self::IssuanceDate<'a> {
			T::issuance_date(*self, context)
		}
		fn issuer<'a>(&'a self, context: &'a C) -> Option<Self::Issuer<'a>> {
			T::issuer(*self, context)
		}
		fn evidence<'a>(&'a self, context: &'a C) -> Self::Evidences<'a> {
			T::evidence(*self, context)
		}
		fn credential_status<'a>(&'a self, context: &'a C) -> Option<Self::CredentialStatus<'a>> {
			T::credential_status(*self, context)
		}
		fn expiration_date<'a>(&'a self, context: &'a C) -> Option<Self::ExpirationDate<'a>> {
			T::expiration_date(*self, context)
		}
		fn terms_of_use<'a>(&'a self, context: &'a C) -> Self::TermsOfUses<'a> {
			T::terms_of_use(*self, context)
		}
	}
	#[doc = " Verifiable Credential."]
	#[doc = ""]
	#[doc = "A set of one or more claims made by an issuer."]
	#[doc = " A verifiable credential is a tamper-evident credential that has authorship"]
	#[doc = " that can be cryptographically verified. Verifiable credentials can be used"]
	#[doc = " to build verifiable presentations, which can also be cryptographically"]
	#[doc = " verified. The claims in a credential can be about different subjects."]
	#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
	pub struct VerifiableCredential<I> {
		#[doc = " Refreshing."]
		#[doc = ""]
		#[doc = "One or more refresh services that provides enough information to the"]
		#[doc = " recipient's software such that the recipient can refresh the verifiable"]
		#[doc = " credential."]
		#[doc = " "]
		#[doc = " It is useful for systems to enable the manual or automatic refresh of"]
		#[doc = " an expired verifiable credential.  The issuer can include the refresh"]
		#[doc = " service as an element inside the verifiable credential if it is intended"]
		#[doc = " for either the verifier or the holder (or both), or inside the"]
		#[doc = " verifiable presentation if it is intended for the holder only. In the"]
		#[doc = " latter case, this enables the holder to refresh the verifiable"]
		#[doc = " credential before creating a verifiable presentation to share with a"]
		#[doc = " verifier. In the former case, including the refresh service inside the"]
		#[doc = " verifiable credential enables either the holder or the verifier to"]
		#[doc = " perform future updates of the credential."]
		#[doc = " "]
		#[doc = " The refresh service is only expected to be used when either the"]
		#[doc = " credential has expired or the issuer does not publish credential status"]
		#[doc = " information. Issuers are advised not to put the `refreshService`"]
		#[doc = " property in a verifiable credential that does not contain public"]
		#[doc = " information or whose refresh service is not protected in some way."]
		pub refresh_service: std::collections::BTreeSet<super::rdfs::Resource<I>>,
		#[doc = " Data Schemas."]
		#[doc = ""]
		#[doc = "One or more data schemas that provide verifiers with enough information"]
		#[doc = " to determine if the provided data conforms to the provided schema."]
		#[doc = " "]
		#[doc = " Data schemas are useful when enforcing a specific structure on a given"]
		#[doc = " collection of data. There are at least two types of data schemas that"]
		#[doc = " this specification considers:"]
		#[doc = "   - Data verification schemas, which are used to verify that the"]
		#[doc = "     structure and contents of a credential or verifiable credential"]
		#[doc = "     conform to a published schema."]
		#[doc = "   - Data encoding schemas, which are used to map the contents of a"]
		#[doc = "     verifiable credential to an alternative representation format, such"]
		#[doc = "     as a binary format used in a zero-knowledge proof."]
		#[doc = " "]
		#[doc = " It is important to understand that data schemas serve a different"]
		#[doc = " purpose from the @context property, which neither enforces data"]
		#[doc = " structure or data syntax, nor enables the definition of arbitrary"]
		#[doc = " encodings to alternate representation formats."]
		pub credential_schema: std::collections::BTreeSet<super::rdfs::Resource<I>>,
		#[doc = " Evidence."]
		#[doc = ""]
		#[doc = "One or more evidence schemes providing enough information for a verifier"]
		#[doc = " to determine whether the evidence gathered by the issuer meets its"]
		#[doc = " confidence requirements for relying on the credential."]
		#[doc = " "]
		#[doc = " Evidence can be included by an issuer to provide the verifier with"]
		#[doc = " additional supporting information in a verifiable credential."]
		#[doc = " This could be used by the verifier to establish the confidence with"]
		#[doc = " which it relies on the claims in the verifiable credential."]
		#[doc = " "]
		#[doc = " For example, an issuer could check physical documentation provided by"]
		#[doc = " the subject or perform a set of background checks before issuing the"]
		#[doc = " credential. In certain scenarios, this information is useful to the"]
		#[doc = " verifier when determining the risk associated with relying on a given"]
		#[doc = " credential."]
		pub evidence: std::collections::BTreeSet<super::rdfs::Resource<I>>,
		#[doc = " Terms of Use."]
		#[doc = ""]
		#[doc = "One or more terms of use policies under which the creator issued the"]
		#[doc = " credential or presentation. If the recipient (a holder or verifier) is"]
		#[doc = " not willing to adhere to the specified terms of use, then they do so on"]
		#[doc = " their own responsibility and might incur legal liability if they violate"]
		#[doc = " the stated terms of use."]
		#[doc = " "]
		#[doc = " Terms of use can be utilized by an issuer or a holder to communicate the"]
		#[doc = " terms under which a verifiable credential or verifiable presentation was"]
		#[doc = " issued. The issuer places their terms of use inside the verifiable"]
		#[doc = " credential. The holder places their terms of use inside a verifiable"]
		#[doc = " presentation. This specification defines a termsOfUse property for"]
		#[doc = " expressing terms of use information."]
		#[doc = " "]
		#[doc = " The value of the `termsOfUse` property tells the verifier what actions"]
		#[doc = " it is required to perform (an obligation), not allowed to perform"]
		#[doc = " (a prohibition), or allowed to perform (a permission) if it is to accept"]
		#[doc = " the verifiable credential or verifiable presentation."]
		pub terms_of_use: std::collections::BTreeSet<super::rdfs::Resource<I>>,
		#[doc = " Status."]
		pub credential_status: Option<super::rdfs::Resource<I>>,
		pub expiration_date: Option<super::xsd::DateTime>,
		#[doc = " Proofs (Signatures)."]
		#[doc = ""]
		#[doc = "One or more cryptographic proofs that can be used to detect tampering"]
		#[doc = " and verify the authorship of a credential or presentation."]
		#[doc = " The specific method used for an embedded proof MUST be included using"]
		#[doc = " the type property. "]
		pub proof: std::collections::BTreeSet<super::rdfs::Resource<I>>,
		pub issuance_date: super::xsd::DateTime,
		#[doc = " Issuer."]
		#[doc = ""]
		#[doc = "It is RECOMMENDED that the URI in the issuer or its id be one which,"]
		#[doc = " if dereferenced, results in a document containing machine-readable"]
		#[doc = " information about the issuer that can be used to verify the information"]
		#[doc = " expressed in the credential."]
		pub issuer: Option<super::rdfs::Resource<I>>,
		pub credential_subject: std::collections::BTreeSet<super::rdfs::Resource<I>>,
	}
	impl<I> VerifiableCredential<I> {
		pub fn new(issuance_date: super::xsd::DateTime) -> Self {
			Self {
				refresh_service: Default::default(),
				credential_schema: Default::default(),
				evidence: Default::default(),
				terms_of_use: Default::default(),
				credential_status: Default::default(),
				expiration_date: Default::default(),
				proof: Default::default(),
				issuance_date: issuance_date,
				issuer: Default::default(),
				credential_subject: Default::default(),
			}
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
		for VerifiableCredential<N::Id>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::chrono::DateTime<::chrono::Utc>: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>,
	{
		fn from_rdf<G>(
			namespace: &mut N,
			id: &N::Id,
			graph: &G,
		) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
		where
			G: ::treeldr_rust_prelude::grdf::Graph<
				Subject = N::Id,
				Predicate = N::Id,
				Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
			>,
		{
			Ok(Self {
				refresh_service: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#refreshService"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				credential_schema: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#credentialSchema"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				evidence: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#evidence"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				terms_of_use: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#termsOfUse"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				credential_status: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#credentialStatus"
							),
						)),
					);
					let object = objects.next();
					if objects.next().is_some() {
						panic!("multiples values on functional property")
					}
					match object {
						Some(object) => {
							Some({
								match object { :: treeldr_rust_prelude :: rdf :: Object :: Id (id) => { :: treeldr_rust_prelude :: FromRdf :: from_rdf (namespace , id , graph) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: UnexpectedLiteralValue) }
							})
						}
						None => None,
					}
				},
				expiration_date: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#expirationDate"
							),
						)),
					);
					let object = objects.next();
					if objects.next().is_some() {
						panic!("multiples values on functional property")
					}
					match object {
						Some(object) => Some({
							match object { :: treeldr_rust_prelude :: rdf :: Object :: Literal (lit) => { < :: chrono :: DateTime < :: chrono :: Utc > as :: treeldr_rust_prelude :: rdf :: FromLiteral < V , N >> :: from_literal (namespace , lit) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: ExpectedLiteralValue) }
						}),
						None => None,
					}
				},
				proof: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#proof"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
				issuance_date: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#issuanceDate"
							),
						)),
					);
					match objects.next() {
						Some(object) => {
							if objects.next().is_some() {
								panic!("multiples values on functional property")
							}
							match object { :: treeldr_rust_prelude :: rdf :: Object :: Literal (lit) => { < :: chrono :: DateTime < :: chrono :: Utc > as :: treeldr_rust_prelude :: rdf :: FromLiteral < V , N >> :: from_literal (namespace , lit) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: ExpectedLiteralValue) }
						}
						None => {
							return Err(
								::treeldr_rust_prelude::FromRdfError::MissingRequiredPropertyValue,
							)
						}
					}
				},
				issuer: {
					let mut objects = graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#issuer"
							),
						)),
					);
					let object = objects.next();
					if objects.next().is_some() {
						panic!("multiples values on functional property")
					}
					match object {
						Some(object) => {
							Some({
								match object { :: treeldr_rust_prelude :: rdf :: Object :: Id (id) => { :: treeldr_rust_prelude :: FromRdf :: from_rdf (namespace , id , graph) ? } , _ => return Err (:: treeldr_rust_prelude :: FromRdfError :: UnexpectedLiteralValue) }
							})
						}
						None => None,
					}
				},
				credential_subject: {
					let mut result = ::std::collections::btree_set::BTreeSet::new();
					for object in graph.objects(
						&id,
						&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(namespace.insert(
							::treeldr_rust_prelude::static_iref::iri!(
								"https://www.w3.org/2018/credentials#credentialSubject"
							),
						)),
					) {
						result.insert(match object {
							::treeldr_rust_prelude::rdf::Object::Id(id) => {
								::treeldr_rust_prelude::FromRdf::from_rdf(namespace, id, graph)?
							}
							_ => {
								return Err(
									::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue,
								)
							}
						});
					}
					result
				},
			})
		}
	}
	impl<C: ?Sized, I> super::rdfs::AnyResource<C> for VerifiableCredential<I> {
		type Comment < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Comments < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Type < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Types < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Label < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Labels < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
			::std::iter::empty()
		}
		fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
			::std::iter::empty()
		}
		fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
			::std::iter::empty()
		}
	}
	pub struct VerifiableCredentialTriplesAndValues<'a, I, V> {
		id_: Option<I>,
		refresh_service: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, super::rdfs::Resource<I>>,
			super::rdfs::ResourceTriplesAndValues<'a, I, V>,
			V,
		>,
		credential_schema: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, super::rdfs::Resource<I>>,
			super::rdfs::ResourceTriplesAndValues<'a, I, V>,
			V,
		>,
		evidence: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, super::rdfs::Resource<I>>,
			super::rdfs::ResourceTriplesAndValues<'a, I, V>,
			V,
		>,
		terms_of_use: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, super::rdfs::Resource<I>>,
			super::rdfs::ResourceTriplesAndValues<'a, I, V>,
			V,
		>,
		credential_status: ::treeldr_rust_prelude::rdf::iter::Optional<
			super::rdfs::ResourceTriplesAndValues<'a, I, V>,
		>,
		expiration_date: ::treeldr_rust_prelude::rdf::iter::Optional<
			::treeldr_rust_prelude::rdf::ValuesOnly<
				::treeldr_rust_prelude::rdf::LiteralValue<
					'a,
					::chrono::DateTime<::chrono::Utc>,
					I,
					V,
				>,
			>,
		>,
		proof: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, super::rdfs::Resource<I>>,
			super::rdfs::ResourceTriplesAndValues<'a, I, V>,
			V,
		>,
		issuance_date: ::treeldr_rust_prelude::rdf::ValuesOnly<
			::treeldr_rust_prelude::rdf::LiteralValue<'a, ::chrono::DateTime<::chrono::Utc>, I, V>,
		>,
		issuer: ::treeldr_rust_prelude::rdf::iter::Optional<
			super::rdfs::ResourceTriplesAndValues<'a, I, V>,
		>,
		credential_subject: ::treeldr_rust_prelude::rdf::FlattenTriplesAndValues<
			::std::collections::btree_set::Iter<'a, super::rdfs::Resource<I>>,
			super::rdfs::ResourceTriplesAndValues<'a, I, V>,
			V,
		>,
	}
	impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
		::treeldr_rust_prelude::RdfIterator<N> for VerifiableCredentialTriplesAndValues<'a, N::Id, V>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::chrono::DateTime<::chrono::Utc>: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
	{
		type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
		fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&mut self,
			vocabulary: &mut N,
			generator: &mut G,
		) -> Option<Self::Item> {
			self . credential_subject . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#credentialSubject"))) , value)) } }) . or_else (|| self . issuer . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#issuer"))) , value)) } }) . or_else (|| self . issuance_date . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#issuanceDate"))) , value)) } }) . or_else (|| self . proof . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#proof"))) , value)) } }) . or_else (|| self . expiration_date . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#expirationDate"))) , value)) } }) . or_else (|| self . credential_status . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#credentialStatus"))) , value)) } }) . or_else (|| self . terms_of_use . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#termsOfUse"))) , value)) } }) . or_else (|| self . evidence . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#evidence"))) , value)) } }) . or_else (|| self . credential_schema . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#credentialSchema"))) , value)) } }) . or_else (|| self . refresh_service . next_with (vocabulary , generator) . map (| item | match item { :: treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (triple) } treeldr_rust_prelude :: rdf :: TripleOrValue :: Value (value) => { treeldr_rust_prelude :: rdf :: TripleOrValue :: Triple (:: rdf_types :: Triple (self . id_ . clone () . unwrap () , treeldr_rust_prelude :: rdf_types :: FromIri :: from_iri (vocabulary . insert (:: treeldr_rust_prelude :: static_iref :: iri ! ("https://www.w3.org/2018/credentials#refreshService"))) , value)) } }) . or_else (|| self . id_ . take () . map (:: treeldr_rust_prelude :: rdf_types :: Object :: Id) . map (:: treeldr_rust_prelude :: rdf :: TripleOrValue :: Value)))))))))))
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
		::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for VerifiableCredential<N::Id>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
		::chrono::DateTime<::chrono::Utc>: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
	{
		type TriplesAndValues < 'a > = VerifiableCredentialTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
		fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&'a self,
			namespace: &mut N,
			generator: &mut G,
		) -> Self::TriplesAndValues<'a>
		where
			N::Id: 'a,
			V: 'a,
		{
			VerifiableCredentialTriplesAndValues {
				id_: Some(generator.next(namespace)),
				refresh_service: self
					.refresh_service
					.unbound_rdf_triples_and_values(namespace, generator),
				credential_schema: self
					.credential_schema
					.unbound_rdf_triples_and_values(namespace, generator),
				evidence: self
					.evidence
					.unbound_rdf_triples_and_values(namespace, generator),
				terms_of_use: self
					.terms_of_use
					.unbound_rdf_triples_and_values(namespace, generator),
				credential_status: self
					.credential_status
					.unbound_rdf_triples_and_values(namespace, generator),
				expiration_date: self
					.expiration_date
					.unbound_rdf_triples_and_values(namespace, generator),
				proof: self
					.proof
					.unbound_rdf_triples_and_values(namespace, generator),
				issuance_date: self
					.issuance_date
					.unbound_rdf_triples_and_values(namespace, generator),
				issuer: self
					.issuer
					.unbound_rdf_triples_and_values(namespace, generator),
				credential_subject: self
					.credential_subject
					.unbound_rdf_triples_and_values(namespace, generator),
			}
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N>
		for VerifiableCredential<N::Id>
	where
		N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
	{
		fn into_json_ld(self, namespace: &N) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
			let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
			if !self.refresh_service.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("refreshService".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.refresh_service
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if !self.credential_schema.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("credentialSchema".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.credential_schema
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if !self.evidence.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("evidence".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.evidence
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if !self.terms_of_use.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("termsOfUse".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.terms_of_use
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			if let Some(value) = self.credential_status {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("credentialStatus".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			if let Some(value) = self.expiration_date {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("expirationDate".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			if !self.proof.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("proof".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.proof
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			result.insert(
				::treeldr_rust_prelude::locspan::Meta("issuanceDate".into(), ()),
				::treeldr_rust_prelude::locspan::Meta(
					::treeldr_rust_prelude::IntoJsonLd::into_json_ld(self.issuance_date, namespace),
					(),
				),
			);
			if let Some(value) = self.issuer {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("issuer".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			if !self.credential_subject.is_empty() {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("credentialSubject".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::json_ld::syntax::Value::Array(
							self.credential_subject
								.into_iter()
								.map(|v| {
									::locspan::Meta(
										::treeldr_rust_prelude::IntoJsonLd::into_json_ld(
											v, namespace,
										),
										(),
									)
								})
								.collect(),
						),
						(),
					),
				);
			}
			result.into()
		}
	}
	impl<C: ?Sized + super::rdfs::AnyResourceProvider<I>, I> AnyVerifiableCredential<C>
		for VerifiableCredential<I>
	{
		type CredentialSchema < 'a > = & 'a super :: rdfs :: Resource < I > where Self : 'a , C : 'a ;
		type CredentialSchemas < 'a > = :: std :: collections :: btree_set :: Iter < 'a , super :: rdfs :: Resource < I > > where Self : 'a , C : 'a ;
		type RefreshService < 'a > = & 'a super :: rdfs :: Resource < I > where Self : 'a , C : 'a ;
		type RefreshServices < 'a > = :: std :: collections :: btree_set :: Iter < 'a , super :: rdfs :: Resource < I > > where Self : 'a , C : 'a ;
		type Proof < 'a > = & 'a super :: rdfs :: Resource < I > where Self : 'a , C : 'a ;
		type Proofs < 'a > = :: std :: collections :: btree_set :: Iter < 'a , super :: rdfs :: Resource < I > > where Self : 'a , C : 'a ;
		type CredentialSubject < 'a > = & 'a super :: rdfs :: Resource < I > where Self : 'a , C : 'a ;
		type CredentialSubjects < 'a > = :: std :: collections :: btree_set :: Iter < 'a , super :: rdfs :: Resource < I > > where Self : 'a , C : 'a ;
		type IssuanceDate < 'a > = & 'a super :: xsd :: DateTime where Self : 'a , C : 'a ;
		type Issuer < 'a > = & 'a super :: rdfs :: Resource < I > where Self : 'a , C : 'a ;
		type Evidence < 'a > = & 'a super :: rdfs :: Resource < I > where Self : 'a , C : 'a ;
		type Evidences < 'a > = :: std :: collections :: btree_set :: Iter < 'a , super :: rdfs :: Resource < I > > where Self : 'a , C : 'a ;
		type CredentialStatus < 'a > = & 'a super :: rdfs :: Resource < I > where Self : 'a , C : 'a ;
		type ExpirationDate < 'a > = & 'a super :: xsd :: DateTime where Self : 'a , C : 'a ;
		type TermsOfUse < 'a > = & 'a super :: rdfs :: Resource < I > where Self : 'a , C : 'a ;
		type TermsOfUses < 'a > = :: std :: collections :: btree_set :: Iter < 'a , super :: rdfs :: Resource < I > > where Self : 'a , C : 'a ;
		fn credential_schema<'a>(&'a self, context: &'a C) -> Self::CredentialSchemas<'a> {
			self.credential_schema.iter()
		}
		fn refresh_service<'a>(&'a self, context: &'a C) -> Self::RefreshServices<'a> {
			self.refresh_service.iter()
		}
		fn proof<'a>(&'a self, context: &'a C) -> Self::Proofs<'a> {
			self.proof.iter()
		}
		fn credential_subject<'a>(&'a self, context: &'a C) -> Self::CredentialSubjects<'a> {
			self.credential_subject.iter()
		}
		fn issuance_date<'a>(&'a self, context: &'a C) -> Self::IssuanceDate<'a> {
			&self.issuance_date
		}
		fn issuer<'a>(&'a self, context: &'a C) -> Option<Self::Issuer<'a>> {
			self.issuer.as_ref()
		}
		fn evidence<'a>(&'a self, context: &'a C) -> Self::Evidences<'a> {
			self.evidence.iter()
		}
		fn credential_status<'a>(&'a self, context: &'a C) -> Option<Self::CredentialStatus<'a>> {
			self.credential_status.as_ref()
		}
		fn expiration_date<'a>(&'a self, context: &'a C) -> Option<Self::ExpirationDate<'a>> {
			self.expiration_date.as_ref()
		}
		fn terms_of_use<'a>(&'a self, context: &'a C) -> Self::TermsOfUses<'a> {
			self.terms_of_use.iter()
		}
	}
}
pub mod rdfs {
	pub trait Class<C: ?Sized>: AnyResource<C> {
		type SubClassOf<'a>: 'a + Class<C>
		where
			Self: 'a,
			C: 'a;
		type SubClassOfs<'a>: 'a + Iterator<Item = Self::SubClassOf<'a>>
		where
			Self: 'a,
			C: 'a;
		fn sub_class_of<'a>(&'a self, context: &'a C) -> Self::SubClassOfs<'a>;
	}
	pub trait ClassProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Class> {
		type Class: Class<Self>;
		fn get(&self, id: &I) -> Option<&Self::Class> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Class>>::get(self, id)
		}
	}
	impl<C: ?Sized> Class<C> for ::std::convert::Infallible {
		type SubClassOf < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type SubClassOfs < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn sub_class_of<'a>(&'a self, _context: &'a C) -> Self::SubClassOfs<'a> {
			unreachable!()
		}
	}
	impl<'r, C: ?Sized, T: Class<C>> Class<C> for &'r T {
		type SubClassOf < 'a > = T :: SubClassOf < 'a > where Self : 'a , C : 'a ;
		type SubClassOfs < 'a > = T :: SubClassOfs < 'a > where Self : 'a , C : 'a ;
		fn sub_class_of<'a>(&'a self, context: &'a C) -> Self::SubClassOfs<'a> {
			T::sub_class_of(*self, context)
		}
	}
	pub trait Datatype<C: ?Sized>: Class<C> {}
	pub trait DatatypeProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::Datatype>
	{
		type Datatype: Datatype<Self>;
		fn get(&self, id: &I) -> Option<&Self::Datatype> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Datatype>>::get(self, id)
		}
	}
	impl<C: ?Sized> Datatype<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Datatype<C>> Datatype<C> for &'r T {}
	pub trait Literal<C: ?Sized>: AnyResource<C> {}
	pub trait LiteralProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::Literal>
	{
		type Literal: Literal<Self>;
		fn get(&self, id: &I) -> Option<&Self::Literal> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::Literal>>::get(self, id)
		}
	}
	impl<C: ?Sized> Literal<C> for ::std::convert::Infallible {}
	impl<'r, C: ?Sized, T: Literal<C>> Literal<C> for &'r T {}
	pub trait AnyResource<C: ?Sized> {
		type Comment<'a>: 'a + Literal<C>
		where
			Self: 'a,
			C: 'a;
		type Comments<'a>: 'a + Iterator<Item = Self::Comment<'a>>
		where
			Self: 'a,
			C: 'a;
		type Type<'a>: 'a + Class<C>
		where
			Self: 'a,
			C: 'a;
		type Types<'a>: 'a + Iterator<Item = Self::Type<'a>>
		where
			Self: 'a,
			C: 'a;
		type Label<'a>: 'a + Literal<C>
		where
			Self: 'a,
			C: 'a;
		type Labels<'a>: 'a + Iterator<Item = Self::Label<'a>>
		where
			Self: 'a,
			C: 'a;
		fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a>;
		fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a>;
		fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a>;
	}
	pub trait AnyResourceProvider<I: ?Sized>:
		::treeldr_rust_prelude::Provider<I, Self::AnyResource>
	{
		type AnyResource: AnyResource<Self>;
		fn get(&self, id: &I) -> Option<&Self::AnyResource> {
			<Self as ::treeldr_rust_prelude::Provider<I, Self::AnyResource>>::get(self, id)
		}
	}
	impl<C: ?Sized> AnyResource<C> for ::std::convert::Infallible {
		type Comment < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Comments < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Type < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Types < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Label < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Labels < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn comment<'a>(&'a self, _context: &'a C) -> Self::Comments<'a> {
			unreachable!()
		}
		fn type_<'a>(&'a self, _context: &'a C) -> Self::Types<'a> {
			unreachable!()
		}
		fn label<'a>(&'a self, _context: &'a C) -> Self::Labels<'a> {
			unreachable!()
		}
	}
	impl<'r, C: ?Sized, T: AnyResource<C>> AnyResource<C> for &'r T {
		type Comment < 'a > = T :: Comment < 'a > where Self : 'a , C : 'a ;
		type Comments < 'a > = T :: Comments < 'a > where Self : 'a , C : 'a ;
		type Type < 'a > = T :: Type < 'a > where Self : 'a , C : 'a ;
		type Types < 'a > = T :: Types < 'a > where Self : 'a , C : 'a ;
		type Label < 'a > = T :: Label < 'a > where Self : 'a , C : 'a ;
		type Labels < 'a > = T :: Labels < 'a > where Self : 'a , C : 'a ;
		fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
			T::comment(*self, context)
		}
		fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
			T::type_(*self, context)
		}
		fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
			T::label(*self, context)
		}
	}
	#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
	pub struct Resource<I> {
		pub id: Option<::treeldr_rust_prelude::Id<I>>,
	}
	impl<C: ?Sized> AnyResource<C> for ::iref::IriBuf {}
	impl<C: ?Sized> AnyResource<C> for ::std::string::String {}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V>
		for Resource<N::Id>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		fn from_rdf<G>(
			namespace: &mut N,
			id: &N::Id,
			graph: &G,
		) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
		where
			G: ::treeldr_rust_prelude::grdf::Graph<
				Subject = N::Id,
				Predicate = N::Id,
				Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
			>,
		{
			Ok(Self {
				id: { Some(::treeldr_rust_prelude::Id(id.clone())) },
			})
		}
	}
	pub struct ResourceTriplesAndValues<'a, I, V> {
		id_: Option<I>,
		_v: ::std::marker::PhantomData<&'a V>,
	}
	impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
		::treeldr_rust_prelude::RdfIterator<N> for ResourceTriplesAndValues<'a, N::Id, V>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type Item = ::treeldr_rust_prelude::rdf::TripleOrValue<N::Id, V>;
		fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&mut self,
			vocabulary: &mut N,
			generator: &mut G,
		) -> Option<Self::Item> {
			self.id_
				.take()
				.map(::treeldr_rust_prelude::rdf_types::Object::Id)
				.map(::treeldr_rust_prelude::rdf::TripleOrValue::Value)
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
		::treeldr_rust_prelude::rdf::TriplesAndValues<N, V> for Resource<N::Id>
	where
		N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
		N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
	{
		type TriplesAndValues < 'a > = ResourceTriplesAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
		fn unbound_rdf_triples_and_values<'a, G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
			&'a self,
			namespace: &mut N,
			generator: &mut G,
		) -> Self::TriplesAndValues<'a>
		where
			N::Id: 'a,
			V: 'a,
		{
			ResourceTriplesAndValues {
				id_: Some(
					self.id
						.clone()
						.map(::treeldr_rust_prelude::Id::unwrap)
						.unwrap_or_else(|| generator.next(namespace)),
				),
				_v: ::std::marker::PhantomData,
			}
		}
	}
	impl<C: ?Sized + AnyResourceProvider<I>, I> AnyResource<C> for Resource<I> {
		type Comment < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Comments < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Type < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Types < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		type Label < 'a > = :: std :: convert :: Infallible where Self : 'a , C : 'a ;
		type Labels < 'a > = :: std :: iter :: Empty < :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
		fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
			::std::iter::empty()
		}
		fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
			::std::iter::empty()
		}
		fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
			::std::iter::empty()
		}
	}
	impl<N: ::treeldr_rust_prelude::rdf_types::Namespace> ::treeldr_rust_prelude::IntoJsonLd<N>
		for Resource<N::Id>
	where
		N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
	{
		fn into_json_ld(self, namespace: &N) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
			let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
			if let Some(value) = self.id {
				result.insert(
					::treeldr_rust_prelude::locspan::Meta("id".into(), ()),
					::treeldr_rust_prelude::locspan::Meta(
						::treeldr_rust_prelude::IntoJsonLd::into_json_ld(value, namespace),
						(),
					),
				);
			}
			result.into()
		}
	}
}
