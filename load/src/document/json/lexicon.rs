use crate::source;

// impl From<treeldr_json_schema::import::Error<source::Metadata>> for LangError {
// 	fn from(e: treeldr_json_schema::import::Error<source::Metadata>) -> Self {
// 		Self::JsonSchema(e)
// 	}
// }

pub fn import(json: json_syntax::MetaValue<source::Metadata>) -> treeldr_lexicon::LexiconDoc {
	json_syntax::from_meta_value(json).expect("invalid Lexicon schema")
}
