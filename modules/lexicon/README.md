# Lexicon module for TreeLDR

This library provides support for the [Lexicon](https://atproto.com/guides/lexicon) schema definition language defined by [The AT Protocol](https://atproto.com/). It allows one to convert any Lexicon schema into a TreeLDR layout that can then be exported into a type definition in the targeted language (e.g. Rust).

For instance, consider the following Lexicon document:
```json
{
  "lexicon": 1,
  "id": "com.example.getProfile",
  "defs": {
    "main": {
      "type": "query",
      "parameters": {
        "user": {"type": "string", "required": true}
      },
      "output": {
        "encoding": "application/json",
        "schema": {
          "type": "object",
          "required": ["did", "name"],
          "properties": {
            "did": {"type": "string"},
            "name": {"type": "string"},
            "displayName": {"type": "string", "maxLength": 64},
            "description": {"type": "string", "maxLength": 256}
          }
        }
      }
    }
  }
}
```
This documents defines an XRPC query schema (`com.example.getProfile`) with sub schemas for the inputs and outputs of the query.

Using the TreeLDR+Rust command line interface one can use the given command to generate the following Rust code:
```
treeldr-rust-cli --no-rdf -i lexicon_doc.json -m example="lexicon:com.example" | rustfmt
```
The `--no-rdf` flag is important to prevent the Rust code generator to generate code related to RDF type definitions (there are none). The output should have the following structure:
```rust
pub struct GetProfile {
  user: get_profile::User;
}

pub mod get_profile {
  pub type User = String;

  pub mod output {
    pub type Did = String;
    pub type Name = String;
    pub struct DisplayName(...);
    pub struct Description(...);
  }

  pub struct Output {
    did: output::Did,
    name: output::Name,
    display_name: Option<output::DisplayName>,
    description: Option<output::Description>
  }
}
```
Each Lexicon schema defined in the document has an equivalent Rust type definition. The generated code is dependent on the `treeldr_rust_prelude` crate.

Unconstrained primitive types such as `User`, `Did` or `Name` (which are all simple strings here) are defined using type aliases. Constrained primitive types such as `DisplayName` and `Description` (that both have a `maxLength` constraint) are defined using opaque structures with a constructor that validates the input value according to the given constraints.
```rust
get_profile::output::DisplayName::new("John Snow".to_string()).except("invalid display name")
```

## Using `treeldr_rust_macros`

Instead of using the command line interface, it is possible to embed the generated code directly inside an existing Rust file using the `tldr` macro provided by the `treeldr_rust_macros` library.
```rust
use treeldr_rust_macros::tldr;

#[tldr("path/to/lexicon_doc.json", no_rdf)]
pub mod schema {
  #[prefix("lexicon:com.example")]
  pub mod example {}
}

schema::example::get_profile::output::DisplayName::new("John Snow".to_string()).except("invalid display name")
```

## Missing features

The Lexicon support is currently incomplete:
  - [Tokens](https://atproto.com/guides/lexicon#tokens) are not supported. No code will be generated for them.
  - Not all datatype constraints are supported. TreeLDR will emit a warning when that occurs. Code will still be generated, but it will be unable to validate the data values according to the unsupported constraints.

The following table summarize the state of datatype constraints support.

Datatype  | Constraint  | Support
--------- | ----------- | -------
boolean   | const       | no
boolean   | default     | no
integer   | const       | no
integer   | default     | no
integer   | enum        | no
integer   | minimum     | yes
integer   | maximum     | yes
string    | const       | no
string    | default     | no
string    | enum        | no
string    | format      | no
string    | minLength   | yes
string    | maxLength   | yes
string    | minGrapheme | yes*
string    | maxGrapheme | yes*
blob      | accept      | no
blob      | maxSize     | no
array     | minLength   | no
array     | maxLength   | no
object    | required    | yes
object    | nullable    | no
record    | key         | no
ref-union | closed      | no

*: The `minGrapheme` and `maxGrapheme` constraints require the `unicode-segmentation` feature of the `treeldr_rust_prelude` crate to be enabled.