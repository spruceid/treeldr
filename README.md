# TreeLDR

TreeLDR is a functional data transformation language based on the Resource Description Framework (RDF). It can be used to serialize RDF datasets into algebraic data values, and back. What makes TreeLDR unique is that functions are **always reversible**: writing a serialization function gives the deserialization function for free.

Here is an example is a very simple TreeLDR function:
```tldr
prefix schema = <https://schema.org/>;

let (foo input) = {
	"id": (id_of input),
	"name": (string_of (get schema:name input)),
	"birth_date": (date_of (get schema:birthDate input))
}
```

In this example we start with defining an IRI prefix so that IRIs of the form `https://schema.org/something` can be shorthanded into `schema:something`. Then we define a function called `foo` with a single `input` parameter. This parameter represents an RDF resource in a dataset. We use it to build a map (a.k.a a JSON object) with a few entries derived from `input`.
- The `"id"` entry is set to the identifier (IRI) of the input value.
- The `"name"` entry is set to the text string representation of the XSD string literal value of the `https://schema.org/name` property of `input`.
- Lastly the `"birth_date"` entry is set to the text string representation of the XSD date literal value of the `https://schema.org/birthDate` property of `input`.

Now let's consider the following RDF dataset:
```n-triples
<https://example.org/johnSmith> <https://schema.org/name> "John Smith" .
<https://example.org/johnSmith> <https://schema.org/birthDate> "2024-12-03"^^<http://www.w3.org/2001/XMLSchema#date> .
```

If we evaluate `foo` using `<https://example.org/johnSmith>` as input value, here is the output we will get:
```json
{
	"id": "https://example.org/johnSmith",
	"name": "John Smith",
	"birth_date": "2024-12-03"
}
```
Because all the functions in TreeLDR are *reversible*, we can also feed this JSON object back to the inverse of `foo` and find the original RDF dataset.

## Command line interface

The top-level package provides a command-line interface to compile and run TreeLDR programs.

### Install

You will need [Rust](https://rust-lang.org) 1.74 or later to build and install the command line interface from the sources, with [cargo](https://doc.rust-lang.org/cargo/).

Start by cloning the git repository:
```console
$ git clone https://github.com/spruceid/treeldr.git
```

You can then build everything from the repository root:
```console
$ cargo build
```

Alternatively if you want to install the binary on your machine, use
```console
$ cargo install --path .
```

### Usage

```console
$ cat path/to/dataset.nq | tldr path/to/program.tldr call arg1 ... argN
```
You can specify the function you want to call using `-f function_name`. By default the `_` function will be called.

Use the `-r` (`--reverse`) flag to call the inverse function.
```console
$ cat tldr path/to/program.tldr call -r arg1 ... argN
```
If you don't provide enough input arguments, TreeLDR will try to read the missing ones from the standard input. For instance:
```console
$ echo '{"id": "http://example.org/#bob", "name": "Bob"}' | tldr examples/simple.tldr call -r
<http://example.org/#bob>
```

### Supported formats

| Tree format | Option value(s)                                  |
| ----------- | ------------------------------------------------ |
| JSON        | `application/json`, `json`                       |
| CBOR        | `application/cbor`, `cbor`                       |

| RDF format  | Option value(s)                                  |
| ----------- | ------------------------------------------------ |
| N-Quads     | `application/n-quads`, `n-quads`, `nquads`, `nq` |

## Library

TreeLDR can be used as a Rust library and embedded in your application. Just add it as a dependency in your `Cargo.toml` file:
```toml
[dependencies]
treeldr = { git = "https://github.com/spruceid/treeldr.git" }
```

Here is a simple example building and evaluating a TreeLDR program:
```rust
// TODO
```

## Testing

To run all the tests, use the following command:
```console
$ cargo test
```