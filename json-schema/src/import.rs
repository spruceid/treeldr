//! JSON Schema import functions.
//! 
//! Semantics follows <https://www.w3.org/2019/wot/json-schema>.
use serde_json::{
	Value
};
use locspan::{
	Location,
	Span,
	Loc
};
use iref::IriBuf;
use rdf_types::{
	Quad
};
use treeldr::{
	vocab,
	Vocabulary,
	Id
};
use vocab::{
	Object,
	LocQuad,
	Name
};

/// Import error.
pub enum Error {
	InvalidJson(serde_json::error::Error),
	InvalidSchema,
	InvalidVocabularyValue,
	InvalidSchemaValue,
	InvalidIdValue,
	InvalidRefValue,
	UnknownKey(String),
	InvalidProperties
}

impl From<serde_json::error::Error> for Error {
	fn from(e: serde_json::error::Error) -> Self {
		Self::InvalidJson(e)
	}
}

/// Create a dummy location.
fn loc<F: Clone>(file: &F) -> Location<F> {
	Location::new(file.clone(), Span::default())
}

pub fn import<F: Clone>(content: &str, file: F, vocabulary: &mut Vocabulary, quads: &mut Vec<LocQuad<F>>) -> Result<(), Error> {
	let schema = serde_json::from_str(content)?;

	import_schema(&schema, &file, vocabulary, quads);

	Ok(())
}

pub fn import_schema<F: Clone>(
	schema: &Value,
	file: &F,
	vocabulary: &mut Vocabulary,
	quads: &mut Vec<LocQuad<F>>
) -> Result<Object<F>, Error> {
	let schema = schema.as_object().ok_or(Error::InvalidSchema)?;

	if let Some(uri) = schema.get("$schema") {
		let uri = uri.as_str().ok_or(Error::InvalidVocabularyValue)?;
	}

	if let Some(object) = schema.get("$vocabulary") {
		let object = object.as_object().ok_or(Error::InvalidVocabularyValue)?;
		
		for (uri, required) in object {
			let required = required.as_bool().ok_or(Error::InvalidVocabularyValue)?;
			todo!()
		}
	}

	let mut is_ref = false;
	let id = match schema.get("$id") {
		Some(id) => {
			let id = id.as_str().ok_or(Error::InvalidIdValue)?;
			let iri = IriBuf::new(id).map_err(|_| Error::InvalidIdValue)?;
			Id::Iri(vocab::Name::from_iri(iri, vocabulary))
		},
		None => match schema.get("$ref") {
			Some(iri) => {
				is_ref = true;
				let iri = iri.as_str().ok_or(Error::InvalidRefValue)?;
				let iri = IriBuf::new(iri).map_err(|_| Error::InvalidRefValue)?;
				Id::Iri(vocab::Name::from_iri(iri, vocabulary))
			}
			None => {
				Id::Blank(vocabulary.new_blank_label())
			}
		}
	};

	// Declare the layout.
	if !is_ref {
		quads.push(Loc(
			Quad(
				Loc(id, loc(file)),
				Loc(Name::Rdf(vocab::Rdf::Type), loc(file)),
				Loc(Object::Iri(Name::TreeLdr(vocab::TreeLdr::Layout)), loc(file)),
				None
			),
			loc(file)
		));
	}

	for (key, value) in schema {
		match key.as_str() {
			"$ref" => (),
			"$dynamicRef" => {
				todo!()
			}
			"$comment" => (),
			"$defs" => {
				todo!()
			},
			// 10. A Vocabulary for Applying Subschemas
			"allOf" => {
				todo!()
			}
			"anyOf" => {
				todo!()
			}
			"oneOf" => {
				todo!()
			}
			"not" => {
				todo!()
			}
			// 10.2.2. Keywords for Applying Subschemas Conditionally 
			"if" => {
				todo!()	
			}
			"then" => {
				todo!()
			}
			"else" => {
				todo!()
			}
			"dependentSchemas" => {
				todo!()
			}
			// 10.3. Keywords for Applying Subschemas to Child Instances
			// 10.3.1. Keywords for Applying Subschemas to Arrays
			"prefixItems" => {
				todo!()
			}
			"items" => {
				todo!()
			}
			"contains" => {
				todo!()
			}
			// 10.3.2. Keywords for Applying Subschemas to Objects
			"properties" => {
				// The presence of this key means that the schema represents a TreeLDR structure layout.
				let properties = value.as_object().ok_or(Error::InvalidProperties)?;

				// First, we build each field.
				let mut fields: Vec<Loc<Object<F>, F>> = Vec::with_capacity(properties.len());
				for (prop, prop_schema) in properties {
					let prop_label = vocabulary.new_blank_label();
					// <prop> rdf:type treeldr:Field
					quads.push(Loc(
						Quad(
							Loc(Id::Blank(prop_label), loc(file)),
							Loc(Name::Rdf(vocab::Rdf::Type), loc(file)),
							Loc(Object::Iri(Name::TreeLdr(vocab::TreeLdr::Field)), loc(file)),
							None
						),
						loc(file)
					));
					// <prop> treeldr:name <name>
					quads.push(Loc(
						Quad(
							Loc(Id::Blank(prop_label), loc(file)),
							Loc(Name::TreeLdr(vocab::TreeLdr::Name), loc(file)),
							Loc(Object::Literal(vocab::Literal::String(
								Loc(
									prop.to_string().into(),
									loc(file)
								)
							)), loc(file)),
							None
						),
						loc(file)
					));

					let prop_schema = import_schema(prop_schema, file, vocabulary, quads)?;
					// quads.push(Loc(
					// 	Quad(
					// 		Loc(Id::Blank(prop_label), loc(file)),
					// 		Loc(Name::TreeLdr(vocab::TreeLdr::Format), loc(file)),
					// 		Loc(Object::Literal(vocab::Literal::String(
					// 			Loc(
					// 				prop.to_string().into(),
					// 				loc(file)
					// 			)
					// 		)), loc(file)),
					// 		None
					// 	),
					// 	loc(file)
					// ));
					todo!()
				}

				// Then we declare the structure content.
				quads.push(Loc(
					Quad(
						Loc(id, loc(file)),
						Loc(Name::Rdf(vocab::Rdf::Type), loc(file)),
						Loc(Object::Iri(Name::TreeLdr(vocab::TreeLdr::Layout)), loc(file)),
						None
					),
					loc(file)
				));
			}
			"patternProperties" => {
				todo!()
			}
			"additionalProperties" => {
				todo!()
			}
			"propertyNames" => {
				todo!()
			}
			// 11. A Vocabulary for Unevaluated Locations
			// 11.1. Keyword Independence
			"unevaluatedItems" => {
				todo!()
			}
			"unevaluatedProperties" => {
				todo!()
			}
			// Validation
			// 6. A Vocabulary for Structural Validation
			"type" => {
				todo!()
			}
			"enum" => {
				todo!()
			}
			"const" => {
				todo!()
			}
			// 6.2. Validation Keywords for Numeric Instances (number and integer)
			"multipleOf" => {
				todo!()
			}
			"maximum" => {
				todo!()
			}
			"exclusiveMaximum" => {
				todo!()
			}
			"minimum" => {
				todo!()
			}
			"exclusiveMinimum" => {
				todo!()
			}
			// 6.3. Validation Keywords for Strings
			"maxLength" => {
				todo!()
			}
			"minLength" => {
				todo!()
			}
			"pattern" => {
				todo!()
			}
			// 6.4. Validation Keywords for Arrays
			"maxItems" => {
				todo!()
			}
			"minItems" => {
				todo!()
			}
			"uniqueItems" => {
				todo!()
			}
			"maxContains" => {
				todo!()
			}
			"minContains" => {
				todo!()
			}
			// 6.5. Validation Keywords for Objects
			"maxProperties" => {
				todo!()
			}
			"minProperties" => {
				todo!()
			}
			"required" => {
				todo!()
			}
			"dependentRequired" => {
				todo!()
			}
			// 7. Vocabularies for Semantic Content With "format"
			"format" => {
				todo!()
			}
			// 8. A Vocabulary for the Contents of String-Encoded Data
			"contentEncoding" => {
				todo!()
			}
			"contentMediaType" => {
				todo!()
			}
			"contentSchema" => {
				todo!()
			}
			// 9. A Vocabulary for Basic Meta-Data Annotations
			"title" => {
				todo!()
			}
			"description" => {
				todo!()
			}
			"default" => {
				todo!()
			}
			"deprecated" => {
				todo!()
			}
			"readOnly" => {
				todo!()
			}
			"writeOnly" => {
				todo!()
			}
			"examples" => {
				todo!()
			}
			// Unknown Name.
			unknown => {
				return Err(Error::UnknownKey(unknown.to_string()))
			}
		}
	}

	let result = match id {
		Id::Iri(id) => Object::Iri(id),
		Id::Blank(id) => Object::Blank(id)
	};

	Ok(result)
}

fn value_into_object<F: Clone>(file: &F, vocab: &mut Vocabulary, quads: &mut Vec<LocQuad<F>>, value: Value) -> Result<Loc<Object<F>, F>, Error> {
	match value {
		Value::Null => todo!(),
		Value::Bool(true) => Ok(Loc(Object::Iri(vocab::Name::Schema(vocab::Schema::True)), loc(file))),
		Value::Bool(false) => Ok(Loc(Object::Iri(vocab::Name::Schema(vocab::Schema::False)), loc(file))),
		Value::Number(n) => Ok(Loc(
			Object::Literal(
				vocab::Literal::TypedString(
					Loc(n.to_string().into(), loc(file)),
					Loc(vocab::Name::Xsd(vocab::Xsd::Integer), loc(file))
				)
			),
			loc(file)
		)),
		Value::String(s) => Ok(Loc(Object::Literal(vocab::Literal::String(Loc(s.to_string().into(), loc(file)))), loc(file))),
		Value::Array(items) => {
			items.into_iter().try_into_rdf_list(&mut (), vocab, quads, loc(file), |item, _, vocab, quads| {
				value_into_object(file, vocab, quads, item)
			})
		}
		Value::Object(_) => todo!()
	}
}

pub trait TryIntoRdfList<F, C, T> {
	fn try_into_rdf_list<E, K>(
		self,
		ctx: &mut C,
		vocab: &mut Vocabulary,
		quads: &mut Vec<LocQuad<F>>,
		loc: Location<F>,
		f: K,
	) -> Result<Loc<Object<F>, F>, E>
	where
		K: FnMut(T, &mut C, &mut Vocabulary, &mut Vec<LocQuad<F>>) -> Result<Loc<Object<F>, F>, E>;
}

impl<F: Clone, C, I: DoubleEndedIterator> TryIntoRdfList<F, C, I::Item> for I {
	fn try_into_rdf_list<E, K>(
		self,
		ctx: &mut C,
		vocab: &mut Vocabulary,
		quads: &mut Vec<LocQuad<F>>,
		loc: Location<F>,
		mut f: K,
	) -> Result<Loc<Object<F>, F>, E>
	where
		K: FnMut(I::Item, &mut C, &mut Vocabulary, &mut Vec<LocQuad<F>>) -> Result<Loc<Object<F>, F>, E>,
	{
		use vocab::Rdf;
		let mut head = Loc(Object::Iri(Name::Rdf(Rdf::Nil)), loc);
		for item in self.rev() {
			let item = f(item, ctx, vocab, quads)?;
			let item_label = vocab.new_blank_label();
			let item_loc = item.location().clone();
			let list_loc = head.location().clone().with(item_loc.span());

			quads.push(Loc(
				Quad(
					Loc(Id::Blank(item_label), list_loc.clone()),
					Loc(Name::Rdf(Rdf::Type), list_loc.clone()),
					Loc(Object::Iri(Name::Rdf(Rdf::List)), list_loc.clone()),
					None,
				),
				item_loc.clone(),
			));

			quads.push(Loc(
				Quad(
					Loc(Id::Blank(item_label), item_loc.clone()),
					Loc(Name::Rdf(Rdf::First), item_loc.clone()),
					item,
					None,
				),
				item_loc.clone(),
			));

			quads.push(Loc(
				Quad(
					Loc(Id::Blank(item_label), head.location().clone()),
					Loc(Name::Rdf(Rdf::Rest), head.location().clone()),
					head,
					None,
				),
				item_loc.clone(),
			));

			head = Loc(Object::Blank(item_label), list_loc);
		}

		Ok(head)
	}
}