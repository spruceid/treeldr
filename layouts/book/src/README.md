# TreeLDR Layouts

TreeLDR Layouts are a data serialization and deserialization tool for the
Resource Description Framework (RDF).
It can be used to convert RDF graphs into tree-like values (such as JSON),
and back.
The idea behind layouts is simple: each layout describes the expected shape of a
tree value.
This shape can be either a record (sometimes called object, in JSON for
instance), a list, a number, etc. Each part of this shape is then associated to
a subset of the represented RDF dataset.

## Basic layout

Here is an example is a very simple TreeLDR layout:
```json
{
	"type": "record", // The shape of the layout (a record here).
	"fields": { // A description of the record's fields.
		"name": { // A field called `name`.
			"value": { "type": "string" }, // The `name` value is a string.
		},
		"age": { // A field called `age`
			"value": { "type": "number" }, // The `age` value is a number.
		}
	}
}
```

This layout matches any record value that may contain the fields `name` and
`age`. The layout of `name` is `{ "type": "string" }`, meaning its value must
be a text string. The layout of `age` is `{ "type": "number" }`, meaning its
value must be a number.

Here is an example of a tree value (here in JSON) matching this layout:
```json
{
	"name": "John Smith",
	"age": 30
}
```

## Adding RDF to the Layout

TreeLDR layouts are meant to define a transformation between tree values and
RDF datasets.
The layout above is a pattern for tree values, but there is not yet mention 
of RDF.
The simplest way to add RDF information to our layout is to use the `property`
attribute to bind each record field to an RDF property:

```json
{
	"type": "record",
	"fields": {
		"name": {
			"value": { "type": "string" },
			// We bind the name `field` to the <http://example.org/#name> property.
			"property": "http://example.org/#name"
		},
		"age": {
			"value": { "type": "number" },
			// We bind the name `age` to the <http://example.org/#age> property.
			"property": "http://example.org/#age"
		}
	}
}
```

With the `property` attributes this layout now maps each matching tree value to
a unique RDF data.
For example, our previous JSON value `{ "name": "John Smith", "age": 30 }` is
mapped to the following RDF dataset (written in N-Triples here):
```n-triples
_:0 <http://example.org/#name> "John Smith" .
_:0 <http://example.org/#age> "30"^^<http://www.w3.org/2001/XMLSchema#decimal> .
```

Here is a walk-through of this dataset:
  - It contains two triples, one for each field of the original tree value
  - `_:0` is the subject of those triples. It is the RDF resource represented
    by the top-level layout. A layout can represent any number of resources,
	called **input** resources. By default a layout has only one input.
	The next section goes over layout inputs in more details.
  - `"John Smith"` is a text string literal value associated to the property
    `http://example.org/#name`, as prescribed by the layout `name` field.
    In RDF each literal value has a type. Here the type is implicitly the XSD
    datatype `http://www.w3.org/2001/XMLSchema#string`. It is possible to
    manually set this datatype.
  - `"30"` is the literal value associated to the property
    `http://example.org/#age`, as prescribed by the layout `age` field.
	Because the `number` layout was used, the default datatype for this
	literal is `http://www.w3.org/2001/XMLSchema#decimal`. It is possible to
	manually set this datatype.

The layout can be used to go from the RDF form to the structured value form by
**serialization**.
It can also be used in the opposite direction from the structured value form to
the pure RDF form by **deserialization**.

## Layout inputs

TODO