pub mod double;
pub mod float;
pub mod real;
pub mod string;

pub enum Restriction<'a> {
	Real(real::Restriction<'a>),
	Float(float::Restriction),
	Double(double::Restriction),
	String(string::Restriction<'a>),
}

pub enum Restrictions<'a> {
	None,
	Real(real::Iter<'a>),
	Float(float::Iter),
	Double(double::Iter),
	String(string::Iter<'a>),
}

impl<'a> Iterator for Restrictions<'a> {
	type Item = Restriction<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Real(r) => r.next().map(Restriction::Real),
			Self::Float(r) => r.next().map(Restriction::Float),
			Self::Double(r) => r.next().map(Restriction::Double),
			Self::String(r) => r.next().map(Restriction::String),
		}
	}
}
