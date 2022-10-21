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
	t01,
	t02,
	t03,
	t04,
	t05,
	t06,
	t07,
	t08,
	t09,
	t10,
	t11,
	t12,
	t13,
	t14,
	t15,
	t16,
	t17,
	t18,
	t19,
	t20,
	t21,
	t22
}

negative! {
	e01,
	e02
}
