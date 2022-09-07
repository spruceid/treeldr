#[test]
fn parse_abnf_grammar() {
	abnf::rulelist(concat!(
		include_str!("../src/grammar/grammar.abnf"),
		include_str!("../src/grammar/rfc3987.abnf")
	))
	.unwrap();
}
