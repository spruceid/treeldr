/// Checks if the given JSON document is a supported Lexicon document.
pub fn json_is_lexicon_doc<M>(
	json: &json_syntax::Value<M>
) -> bool {
	match json.as_object() {
		Some(object) => {
			match object.get("lexicon").next() {
				Some(value) => match value.as_number() {
					Some(number) => number.as_str() == "1",
					None => false
				},
				None => false
			}
		}
		None => false
	}
}