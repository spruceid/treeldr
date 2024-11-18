use json_syntax::{Parse as ParseJson, TryFromJson};
use nquads_syntax::Parse as ParseNQuads;
use paste::paste;
use rdf_types::dataset::IndexedBTreeDataset;
use rdf_types::{BlankIdBuf, Term};
use static_iref::iri;
use std::fs;
use std::path::{Path, PathBuf};
use treeldr_layouts::layout::LayoutType;
use treeldr_layouts::utils::strip_rdf_quad;
use treeldr_layouts::{Layouts, Ref};

fn file_path(id: &str, suffix: &str) -> PathBuf {
	format!("{}/tests/distill/{id}{suffix}", env!("CARGO_MANIFEST_DIR")).into()
}

fn load_layout(layout_path: &Path) -> (Layouts, Ref<LayoutType>) {
	// Initialize the layout builder.
	let mut builder = treeldr_layouts::abs::Builder::new();

	// Parse the layout definition.
	let raw_json = fs::read_to_string(layout_path).unwrap();
	let (layout_json, layout_code_map) = json_syntax::Value::parse_str(&raw_json).unwrap();
	match treeldr_layouts::abs::syntax::Layout::try_from_json(&layout_json, &layout_code_map) {
		Ok(layout_abs) => {
			// We also test parsing through `serde`.
			if let Err(e) = serde_json::from_str::<treeldr_layouts::abs::syntax::Layout>(&raw_json)
			{
				panic!("layout `serde` parse error: {e}")
			}

			let layout_ref = layout_abs.build(&mut builder).unwrap();

			// Compile the layouts.
			let layouts = builder.build();

			(layouts, layout_ref)
		}
		Err(e) => {
			panic!("layout parse error: {e}")
		}
	}
}

fn hydrate<const N: usize>(id: &str, inputs: [Term; N]) {
	// File paths.
	let input_path = file_path(id, "-in.nq");
	let layout_path = file_path(id, "-layout.json");
	let output_path = file_path(id, "-out.json");

	// Parse the input dataset from N-Quads.
	let dataset: IndexedBTreeDataset =
		nquads_syntax::Document::parse_str(&std::fs::read_to_string(input_path).unwrap())
			.unwrap()
			.into_value()
			.into_iter()
			.map(strip_rdf_quad)
			.collect();

	let (layouts, layout_ref) = load_layout(&layout_path);

	// Parse the expected output.
	let expected_json: serde_json::Value =
		fs::read_to_string(output_path).unwrap().parse().unwrap();
	let expected: treeldr_layouts::Value = expected_json.into();

	// Hydrate.
	let output = treeldr_layouts::distill::hydrate(&layouts, &dataset, &layout_ref, &inputs)
		.unwrap()
		.into_untyped();

	// Test.
	assert_eq!(output, expected)
}

fn dehydrate<const N: usize>(id: &str, expected_values: [Term; N]) {
	// File paths.
	let input_path = file_path(id, "-out.json");
	let layout_path = file_path(id, "-layout.json");
	let output_path = file_path(id, "-in.nq");

	// Parse the JSON input.
	let input_json: serde_json::Value = fs::read_to_string(input_path).unwrap().parse().unwrap();
	let input: treeldr_layouts::Value = input_json.into();

	// Parse the expected output dataset from N-Quads.
	let expected_dataset: IndexedBTreeDataset =
		nquads_syntax::Document::parse_str(&std::fs::read_to_string(output_path).unwrap())
			.unwrap()
			.into_value()
			.into_iter()
			.map(strip_rdf_quad)
			.collect();

	let (layouts, layout_ref) = load_layout(&layout_path);

	// Hydrate.
	let (output_dataset, output_values) = treeldr_layouts::distill::dehydrate(
		&layouts,
		&input,
		&layout_ref,
		treeldr_layouts::distill::de::Options::default()
			.with_input_count(expected_values.len() as u32),
	)
	.unwrap();

	eprintln!("dataset:");
	for quad in &output_dataset {
		eprintln!("{quad} .")
	}

	// Test.
	let bijection =
		rdf_types::dataset::isomorphism::find_bijection(&output_dataset, &expected_dataset)
			.expect("not isomorphic"); // fail if the output is not isomorphic to the expected dataset.

	assert_eq!(output_values.len(), expected_values.len());
	for (output, expected) in output_values.iter().zip(&expected_values) {
		if output.is_blank() {
			assert_eq!(*bijection.forward.get(output).unwrap(), expected)
		} else {
			assert_eq!(output, expected)
		}
	}
}

macro_rules! test {
	($(#[$meta:meta])* $name:ident ($($e:expr),*)) => {
		paste! {
			$(#[$meta])*
			#[test]
			fn [<hydrate_ $name>] () {
				hydrate(stringify!($name), [$($e),*])
			}
		}

		paste! {
			$(#[$meta])*
			#[test]
			fn [<dehydrate_ $name>] () {
				dehydrate(stringify!($name), [$($e),*])
			}
		}
	};
}

macro_rules! negative_test {
	($(#[$meta:meta])* $name:ident ($($e:expr),*)) => {
		paste! {
			$(#[$meta])*
			#[test]
			#[should_panic]
			fn [<hydrate_ $name>] () {
				hydrate(stringify!($name), [$($e),*])
			}
		}

		paste! {
			$(#[$meta])*
			#[test]
			#[should_panic]
			fn [<dehydrate_ $name>] () {
				dehydrate(stringify!($name), [$($e),*])
			}
		}
	};
}

test! {
	/// Simple record layout.
	t01 (Term::blank(BlankIdBuf::new("_:john_smith".to_string()).unwrap()))
}

test! {
	/// Simple compact record layout (equivalent to `t01`).
	t02 (Term::blank(BlankIdBuf::new("_:john_smith".to_string()).unwrap()))
}

test! {
	/// Record layout.
	t03 (Term::blank(BlankIdBuf::new("_:receipt".to_string()).unwrap()))
}

test! {
	/// Compact record layout (equivalent to `t03`).
	t04 (Term::blank(BlankIdBuf::new("_:receipt".to_string()).unwrap()))
}

test! {
	/// Simple list.
	t05 (Term::blank(BlankIdBuf::new("_:list".to_string()).unwrap()))
}

test! {
	/// Compact simple list (equivalent to `t05`).
	t06 (Term::blank(BlankIdBuf::new("_:list".to_string()).unwrap()))
}

test! {
	/// Simple set.
	t07 (Term::blank(BlankIdBuf::new("_:subject".to_string()).unwrap()))
}

test! {
	/// Compact simple set (equivalent to `t07`).
	t08 (Term::blank(BlankIdBuf::new("_:subject".to_string()).unwrap()))
}

test! {
	/// Simple sized list.
	t09 (Term::blank(BlankIdBuf::new("_:subject".to_string()).unwrap()))
}

test! {
	/// Compact simple sized list (equivalent to `t09`).
	t10 (Term::blank(BlankIdBuf::new("_:subject".to_string()).unwrap()))
}

test! {
	/// IRI identifier.
	t11 (Term::iri(iri!("https://example.org/JohnSmith").to_owned()))
}

test! {
	/// Sum layout.
	t12 (Term::blank(BlankIdBuf::new("_:subject".to_string()).unwrap()))
}

test! {
	/// Unit layout with const value.
	t13 (Term::blank(BlankIdBuf::new("_:subject".to_string()).unwrap()))
}

test! {
	/// Layout reference.
	t14 (Term::blank(BlankIdBuf::new("_:subject".to_string()).unwrap()))
}

test! {
	/// Literal value.
	t15 (Term::blank(BlankIdBuf::new("_:subject".to_string()).unwrap()))
}

negative_test! {
	/// Missing required field.
	e01 (Term::blank(BlankIdBuf::new("_:john_smith".to_string()).unwrap()))
}

#[test]
fn dehydrate_t16() {
	dehydrate(
		"t16",
		[Term::blank(
			BlankIdBuf::new("_:subject".to_string()).unwrap(),
		)],
	)
}
