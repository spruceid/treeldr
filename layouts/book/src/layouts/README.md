# Layouts

A layout defines a bidirectional transformation from/to RDF datasets and
tree values (as defined in the [Values](/data-model/values.md) section).
Using a layout to transform an RDF dataset to a value is called *serialization*.
The inverse transformation, from value to RDF dataset, is called
*deserialization*.

TODO illustration

## Inputs

Each layout has a set of inputs specifying which RDF resources are subject to
the transformation.

TODO example

## Variable Introduction

TODO

## Type Definition

```ts
type Layout = Never | LiteralLayout | ProductLayout | SumLayout | ListLayout | Always ;
```