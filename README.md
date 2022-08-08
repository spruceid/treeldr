# TreeLDR

An intuitive Linked Data schema definition language and boilerplate code generator.
TreeLDR can be used to produce JSON Schemas, JSON-LD contexts, migration strategies, blockchain publishing routines, etc.
and entire SDKs in various target programing languages such as Python, Java and more.
This way, developers can define data structures in a familiar way and focus purely on the application level.

## Installation

You will need [Rust](https://rust-lang.org) 1.64 (nightly) or later
to install TreeLDR, with [cargo](https://doc.rust-lang.org/cargo/).
The nightly version is required until
[generic associated types are stabilized](https://github.com/rust-lang/rust/pull/96709).

TreeLDR can be installed from the source by first cloning
the git repository:
```
git clone https://github.com/spruceid/treeldr.git
```

This repository contains the different libraries composing
the TreeLDR language, the compiler library and its modules.
You can then build and install the compiler using `cargo`
from the repository root:
```
cargo +nightly install --path tldrc
```

## User Guide

See the [user guide](https://www.spruceid.dev) for a complete overview
of TreeLDR's features.

## Basic Usage

A treeLDR schema is a collection of types and layouts.
A type defines the semantics of a piece of data,
while a layout defines its structure.
One type can have many layouts,
while each layout is associated to a single type.

Here is a simple example
with the definition of a `Person` type.
```tldr
// Sets the base IRI of the document.
base <https://example.com/>;

// Defines an `xs` prefix for the XML schema datatypes.
use <http://www.w3.org/2001/XMLSchema#> as xs;

/// A person.
type Person {
	/// Full name.
	name: required xs:string,

	/// Parents.
	parent: multiple Person,

	/// Age.
	age: xs:nonNegativeInteger
}
```

### Export

TreeLDR schema definitions can be exported into various
other schema description languages or programming language type
definitions.

```
tldrc -i <input file 1> ... -i <input file N> <target> <layout>
```
where `<target>` is a sub command selecting the target language
(e.g. `json-ld-context`, `json-schema`, etc.)
and `<layout>` the IRI of the layout to export.
SO target languages such as `json-ld-context` allow you to export
multiple layouts at once.
Use `--help` to see the description of each sub command and its options.

#### Generating a JSON-LD context

With the previous example
we can use the following command:
```
tldrc -i example/xsd.tldr -i example/person.tldr json-ld-context https://example.com/Person
```
The `example/xsd.tldr` file should contains all the XSD type definitions.
This will generate the given JSON-LD context:
```json
{
	"name": "https://example.com/Person/name",
	"parent": "https://schema.org/Person/parent",
	"age": "https://schema.org/Person/age"
}
```

#### Generating a JSON Schema

We can also use the following command:
```
tldrc -i example/xsd.tldr -i example/person.tldr json-schema https://example.com/Person
```
This will generate the given JSON Schema for the same layout:
```json
{
	"$schema": "https://json-schema.org/draft/2020-12/schema",
	"$id": "https://example.com/person.schema.json",
	"description": "Person",
	"type": "object",
	"properties": {
		"name": {
			"description": "Full name",
			"type": "string"
		}
		"parent": {
			"description": "Parents",
			"type": "array",
			"item": {
			"$ref": "https://example.com/person.schema.json"
			}
		}
		"age": {
			"description": "Age",
			"type": "integer",
			"minimum": 0
		}
	},
	"required": [
		"name"
	]
}
```

#### Generating a Rust module

TreeLDR provides two Rust libraries
`treeldr_rust_macros` and `treeldr_rust_prelude` that
help integrate TreeLDR with Rust.
The first define a `#[treeldr]` procedural macro attribute
that generates code compatible with type definitions
provided by `treeldr_rust_prelude`.

```rust
#[tldr(
	"examples/xsd.tldr",
	"examples/person.tldr"
)]
pub mod schema {
	/// XSD datatypes.
	#[prefix("http://www.w3.org/2001/XMLSchema#")]
	pub mod xs {}

	/// Person example type.
	#[prefix("https://example.com/")]
	pub mod example {}
}
```

This will expand into the following code:
```rust
pub mod schema {
	pub mod xs {
		pub type String = ::std::alloc::String;
	}

	pub mod example {
		/// A person.
		#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
		pub struct Person {
			/// Full name.
			name: super::xs::String,

			/// Parents.
			parent: BTreeSet<Person>,

			/// Age.
			age: Option<u32>
		}
	}
}
```