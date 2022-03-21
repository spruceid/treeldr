# TreeLDR package manager

The package manager is in charge of resolving/importing dependencies,
calling the `tldrc` compiler with the correct parameters to generate the
desired targets, and publishing new definitions.

## TreeLDR Package

A package is primarily defined by a base IRI and an RDF dataset defining
new terms relative to this base IRI.
For every RDF quad in the dataset:
 - Either the subject, predicate or object **must** be relative to the base IRI.
 - The graph **must** be relative to the base IRI.

### Versioning

A package is published on a registry with a *version* strictly following
[Semantic Versioning 2.0.0](https://semver.org/).

A change in the dataset is considered backward compatible iff:
 - No quad is removed; and
 - The predicate of a newly added quad is **not**
 `http://www.w3.org/1999/02/22-rdf-syntax-ns#type`
 - The predicate is **not** a property taking at most one value
 (declared for instance with `https://schema.org/multipleValues`).

Duplicates are ignored. Implicit quads (derived from the semantics of a term)
are not ignored.
For instance, for every property `p`, the following implicit quad is always
defined:
```n-quads
p <https://schema.org/multipleValues> <https://schema.org/False>
```
unless the following quad is explicitly defined:
```n-quads
p <https://schema.org/multipleValues> <https://schema.org/True>
```
Adding the later would be a breaking change, as it implicitly means removing the
former.

## Registry

Packages are downloaded from a registry, or directly from the source for well known packages such as `https://schema.org`.

## HTTP schemes

Two terms are considered equivalent if every parts of their IRI except for the schemes are equals, and their schemes are either `http` and/or `https`.