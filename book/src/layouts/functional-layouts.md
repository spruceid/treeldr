# Functional Layout

A functional layout is a layout where each tree node matching this layout
represents exactly one RDF resource.
In other words, such layout defines *function* from tree nodes to RDF resources.
A functional layout and all its referenced layouts must have exactly **one** input (`self`).

## Addressing

We can use [tree paths](../data-model/paths.md) to address tree nodes.
In a functional layout, each tree node maps to one RDF resource.
This means we can use the tree paths to address RDF resources (and their layout).