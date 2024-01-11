# Functional Layout

A functional layout is a layout where each tree node matching this layout
represents exactly one RDF resource.
In other words, such layout defines *function* from tree nodes to RDF resources.

## Restrictions

Functional layouts share the following restrictions:
- Must have exactly **one** input (`self`).
- Must have exactly **zero** intros.

In addition, each layout type has its own restrictions defined as follows.

### Restrictions on literal layouts

- `resource` attribute must be `self`.

### Restrictions on product (`record`) layouts

- Each field must represent a single RDF property of `self`.
	- Must have exactly one intro (`value`).
	- The value layout must be functional, called with the input `value`.
	- Must define exactly one triple in `dataset` of the following form:
```
_:self <http://example.org/#someProperty> _:value .
```

### Restrictions on list layouts

- Each list node must have exactly **one** intro (`value`)
- The node value layout must be functional, called with the input `value`.

### Restrictions on sum layouts

- Each variant must have exactly **zero** intros.
- Each variant value layout must be functional, called with the input `self`.

## Addressing

We can use [tree paths](../data-model/paths.md) to address tree nodes.
In a functional layout, each tree node maps to one RDF resource.
This means we can use the tree paths to address RDF resources (and their layout).