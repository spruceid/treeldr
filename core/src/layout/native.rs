#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Native {
	/// Boolean.
	Boolean,

	/// Integer number.
	Integer,

	/// Positive integer number.
	PositiveInteger,

	/// Floating point number.
	Float,

	/// Double.
	Double,

	/// String.
	String,

	/// Time.
	Time,

	/// Date.
	Date,

	/// Date and time.
	DateTime,

	/// IRI.
	Iri,

	/// URI.
	Uri,

	/// URL.
	Url,
}
