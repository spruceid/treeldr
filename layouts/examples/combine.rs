use rdf_types::{dataset::BTreeDataset, BlankIdBuf, Id, RdfDisplay, Term};
use serde_json::json;
use treeldr_layouts::{
	abs::{syntax::Layout, Builder},
	distill::de::Options,
};

fn main() {
	let mut builder = Builder::default();

	let nested_json = json!(
		{
			"nested": { "egg": 1 }
		}
	);

	let unnested_json = json!(
		{
			"unnested": { "bird": true }
		}
	);
	let nested_layout: Layout = serde_json::from_value(json!({
	  "id": "https://example.org#nestedResult",
	  "type": "record",
	  "fields": {
		"nested": {
		  "value": {
			"id": "https://example.org#nested",
			"type": "record",
			"fields": {
			  "egg": {
				"value": {
				  "type": "number",
				  "id": "https://example.org#egg",
				  "datatype": "http://www.w3.org/2001/XMLSchema#nonNegativeInteger"
				},
				"property": "https://example.org#egg"
			  }
			}
		  },
		  "property": "https://example.org#nested"
		}
	  }
	}))
	.unwrap();
	println!("Nested layout unwrapped!");

	let unnested_layout: Layout = serde_json::from_value(json!({
	  "id": "https://example.org#unnestedResult",
	  "type": "record",
	  "fields": {
		"unnested": {
		  "value": {
			"id": "https://example.org#unnested",
			"type": "record",
			"fields": {
			  "bird": {
				"value": {
				  "type": "boolean",
				  "id": "https://example.org#bird"
				},
				"property": "https://example.org#bird"
			  },
			}
		  },
		  "property": "https://example.org#unnested"
		}
	  }
	}))
	.unwrap();
	println!("Unnested layout unwrapped!");

	let final_layout: Layout = serde_json::from_value(json!({
	  "id": "https://example.org#anOutputLayout",
	  "type": "record",
	  "fields": {
		"nested": {
		  "value": "https://example.org#nested",
		  "property": "https://example.org#nested"
		},
		"bird": {
		  "intro": ["unnested", "bird"],
		  "value": {
			"layout": "https://example.org#bird",
			"input": "_:bird"
		  },
		  "dataset": [
			["_:self", "https://example.org#unnested", "_:unnested"],
			["_:unnested", "https://example.org#bird", "_:bird"]
		  ]
		}
	  }
	}))
	.unwrap();
	println!("Final layout unwrapped!");

	let nested_ref = nested_layout.build(&mut builder).unwrap();
	println!("Made nested ref: {nested_ref:?}");

	let unnested_ref = unnested_layout.build(&mut builder).unwrap();
	println!("Made unnested ref: {unnested_ref:?}");

	let final_ref = final_layout.build(&mut builder).unwrap();
	println!("Made final ref: {final_ref:?}");

	let layouts = builder.build();
	println!("Built the layouts");

	let mut generator = rdf_types::generator::Blank::new();

	let mut dataset = BTreeDataset::default();
	let (nested_dataset, _) = treeldr_layouts::distill::de::dehydrate(
		&layouts,
		&nested_json.into(),
		&nested_ref,
		Options::default().with_generator(&mut generator),
	)
	.unwrap();

	dataset.extend(nested_dataset);

	let (unnested_dataset, _) = treeldr_layouts::distill::de::dehydrate(
		&layouts,
		&unnested_json.into(),
		&unnested_ref,
		Options::default().with_generator(&mut generator),
	)
	.unwrap();

	dataset.extend(unnested_dataset);

	println!("Dataset Built:");
	for quad in &dataset {
		println!("{} .", quad.rdf_display());
	}

	let v = treeldr_layouts::hydrate(
		&layouts,
		&dataset,
		&final_ref,
		&[Term::Id(Id::Blank(
			BlankIdBuf::from_suffix("input0").unwrap(),
		))],
	)
	.unwrap()
	.into_untyped();

	println!(
		"Hydrated value:\n{}",
		serde_json::to_string_pretty(&v).unwrap()
	);
}
