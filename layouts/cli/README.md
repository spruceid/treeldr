# TreeLDR Layouts

<!-- cargo-rdme start -->

TreeLDR's RDF Layouts are a powerful tool to map structured data to RDF datasets.
This library provides core types to define layouts, an abstract syntax to
describe layouts and "distillation" functions to serialize/deserialize data
using layouts.

## Basic usage

The following example shows how to create a layout from its abstract syntax
representation (using JSON), compile it and use it to serialize an RDF
dataset into a structured value.

```rust
use static_iref::iri;
use rdf_types::{Quad, Term, Literal, literal::Type};
use xsd_types::XSD_STRING;
use serde_json::json;

// Create a layout builder.
let mut builder = treeldr_layouts::abs::Builder::new();

// Parse the layout definition, here from JSON.
let layout: treeldr_layouts::abs::syntax::Layout = serde_json::from_value(
  json!({
    "type": "record",
    "fields": {
      "id": {
        "value": {
          "layout": { "type": "id" },
          "input": "_:self"
        }
      },
      "name": {
        "value": { "type": "string" },
        "property": "https://schema.org/name"
      }
    }
  })
).unwrap();

// Build the layout.
let layout_ref = layout.build(&mut builder).unwrap(); // returns a `Ref` to the layout.

// Get the compiled layouts collection.
let layouts = builder.build();

// Create an RDF dataset with a single triple.
let dataset: grdf::BTreeDataset = [
  Quad(
    Term::iri(iri!("https://example.org/#john.smith").to_owned()),
    Term::iri(iri!("https://schema.org/name").to_owned()),
    Term::Literal(Literal::new("John Smith".to_owned(), Type::Any(XSD_STRING.to_owned()))),
    None
  )
].into_iter().collect();

// Hydrate the dataset to get a structured data value.
let value = treeldr_layouts::hydrate(
  &layouts,
  &dataset,
  &layout_ref,
  &[Term::iri(iri!("https://example.org/#john.smith").to_owned())]
).unwrap().into_untyped(); // we don't care about types here.

// Create a structured data value with the expected result.
// Parse the layout definition, here from JSON.
let expected: treeldr_layouts::Value = serde_json::from_value(
  json!({
    "id": "https://example.org/#john.smith",
    "name": "John Smith"
  })
).unwrap();

// Check equality.
assert_eq!(value, expected)
```

## The `Layout` types

Layouts come in several forms:
  - `abs::syntax::Layout`: represents a
    layout definition in the abstract syntax. In this representation
    variables have names and layouts can be nested.
  - `abs::Layout`: represents an abstract layout with
    stripped variable names and flattened layouts. These layouts are
    managed by the layout [`Builder`](https://docs.rs/treeldr-layouts/latest/treeldr_layouts/abs/struct.Builder.html).
  - `Layout`: the most optimized and compact form, used
    by the distillation functions. Such layouts are stored in a
    [`Layouts`](https://docs.rs/treeldr-layouts/latest/treeldr_layouts/struct.Layouts.html) collection.

<!-- cargo-rdme end -->
