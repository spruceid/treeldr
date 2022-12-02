use treeldr_syntax::Parse;

pub enum Test {
	Positive { input: &'static str },
	Negative { input: &'static str },
}

impl Test {
	fn run(self) {
		match self {
			Self::Positive { input } => {
				let input = std::fs::read_to_string(input).expect("unable to read input file");
				treeldr_syntax::Document::parse_str(&input, |span| span).expect("parse error");
			}
			Self::Negative { input } => {
				let input = std::fs::read_to_string(input).expect("unable to read input file");
				treeldr_syntax::Document::parse_str(&input, |span| span).expect("parse error");
			}
		}
	}
}

macro_rules! positive {
	{ $($id:ident),* } => {
		$(
			#[test]
			fn $id () {
				Test::Positive {
					input: concat!("tests/parse/", stringify!($id), ".tldr")
				}.run()
			}
		)*
	};
}

macro_rules! negative {
	{ $($id:ident),* } => {
		$(
			#[test]
			#[should_panic]
			fn $id () {
				Test::Negative {
					input: concat!("tests/parse/", stringify!($id), ".tldr")
				}.run()
			}
		)*
	};
}

positive! {
	p001,
	p002,
	p003,
	p004,
	p005,
	p006,
	p007,
	p008,
	p009,
	p010,
	p011,
	p012,
	p013,
	p014,
	p015,
	p016,
	p017,
	p018,
	p019,
	p020,
	p021,
	p022
}

negative! {
	n001,
	n002
}
