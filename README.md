# TreeLDR

TreeLDR is Linked-Data serialization framework providing:
  - RDF layouts to serialize and deserialize RDF datasets from/to tree data;
  - A concise schema definition language for RDF classes and layouts;
  - A set of boilerplate code generator.

TreeLDR can be used to produce JSON Schemas, JSON-LD contexts, migration
strategies, blockchain publishing routines, etc. and entire SDKs in various
target programming languages such as Python, Java and more. This way, developers
can define data structures in a familiar way and focus purely on the application
level.

> [!IMPORTANT]
> TreeLDR has experienced major changes with version 0.2. In particular, we
> formalized the data model of Layouts, changing their definition in a way that
> is incompatible with our current implementation of the DSL and (some)
> generators. We will reintroduce the missing DSL and generators in future
> updates.

## Install

You will need [Rust](https://rust-lang.org) 1.74 or later
to install TreeLDR, with [cargo](https://doc.rust-lang.org/cargo/).

TreeLDR can be installed from the source by first cloning
the git repository:
```console
$ git clone https://github.com/spruceid/treeldr.git
```

This repository contains the different libraries and executables composing
the TreeLDR framework.
You can then build everything from the repository root:
```console
$ cargo build
```

Alternatively if you want to install the binaries on you computer, use
```console
$ cargo install --path .
```

## Usage

The top-level package provides a command-line interface which can use TreeLDR
layouts to serialize or deserialize tree value (like JSON), or generate code.
If you want to use TreeLDR layouts directly in your code, use the
[`treeldr-layouts` library](layouts).

In this section, the `tldr` command can be replaced with `cargo run -- ` if
you chose not to install the binary.

### Deserialization

Use the `dehydrate` subcommand to turn any *tree value* (JSON) into an RDF
dataset using a given layout.
The input tree is read from the standard input, and the output written to the
standard output.

```console
$ tldr path/to/layout.json dehydrate
```

Example layouts are found in the `layouts/examples` folder.
```console
$ echo '{"id": "http://example.org/#bob", "name": "Bob"}' | tldr layouts/examples/record.json dehydrate
<http://example.org/#bob> <https://schema.org/name> "Bob" .
```

You can specify the input (tree) format using the `-i` option after `dehydrate`.
Similarly, you can specify the output (RDF) format using the `-o` option.
By default the input is expected to be JSON and output is N-Quads.
Supported formats are given in the [Supported Formats](#supported-formats)
section below.

### Serialization

Use `hydrate` subcommand to turn any *RDF dataset* into a tree value (JSON)
using a given layout.
The input dataset is read from the standard input, and the output written to the
standard output.

```console
$ tldr path/to/layout.json hydrate
```

Example layouts are found in the `layouts/examples` folder.
```console
$ echo '<http://example.org/#bob> <https://schema.org/name> "Bob" .' | tldr layouts/examples/record.json hydrate 'http://example.org/#bob'
{"id":"http://example.org/#bob","name":"Bob"}
```

You can specify the input (RDF) format using the `-i` option after `hydrate`.
Similarly, you can specify the output (tree) format using the `-o` option.
By default the input is expected to be N-Quads and output is JSON.
Supported formats are given in the [Supported Formats](#supported-formats)
section below.

### Supported formats

The following table lists all the tree formats supported by TreeLDR.
The "Option value" can be given to the `-i` option of the `dehydrate`
subcommand, or the `-o` option of the `hydrate` subcommand.

| Tree format | Option value(s)                                  |
| ----------- | ------------------------------------------------ |
| JSON        | `application/json`, `json`                       |
| CBOR        | `application/cbor`, `cbor`                       |

The following table lists all the RDF formats supported by TreeLDR.
The "Option value" can be given to the `-i` option of the `hydrate` subcommand,
or the `-i` option of the `dehydrate` subcommand.

| RDF format  | Option value(s)                                  |
| ----------- | ------------------------------------------------ |
| N-Quads     | `application/n-quads`, `n-quads`, `nquads`, `nq` |

## Tesing

To run all the tests, use the following command:
```console
$ cargo test --workspace --all-features
```