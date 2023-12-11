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

## Installation

You will need [Rust](https://rust-lang.org) 1.70 or later
to install TreeLDR, with [cargo](https://doc.rust-lang.org/cargo/).

TreeLDR can be installed from the source by first cloning
the git repository:
```
git clone https://github.com/spruceid/treeldr.git
```

This repository contains the different libraries and executables composing
the TreeLDR framework.
You can then build everything from the repository root:
```
cargo build
```