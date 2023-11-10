# TreeLDR Layouts

TreeLDR Layouts are a data serialization and deserialization tool for the
Resource Description Framework (RDF) used to put RDF datasets in structured
form, and back.
The idea behind layouts is simple: each layout describes the expected shape of a
structured value. This shape can be either a record (sometimes called object,
in JSON for instance), a list, a number, etc. Each part of this shape is then
associated to a fraction of the represented RDF dataset.

Here is a layout example:

```json
{
	"type": "record",
	"fields": {
		"id": {
			"value": {
				"layout": { "type": "id" },
				"input": "_:self"
			},
		},
		"name": {
			"value": { "type": "string" },
			"dataset": [
				["_:self", "http://example.org/#name", "_:value"]
			]
		}
	}
}
```

This layout recognizes every record value that may contain the fields `id` and
`name`. The layout of the `id` field is an identifier (`"type": "id"`)
identifying `_:self`, the *subject* of the layout. The layout of the `name`
field is a text string literal (`"type": "string"`) and is associated to a
dataset fragment setting the name of the subject to the field's value using
the `http://example.org/#name` RDF property.

Here is an example of a structured value (here in JSON) matching this layout:
```json
{
	"id": "http://example.org/#john",
	"name": "John Smith"
}
```

Here is the same example in pure RDF form (written in N-Triples here):
```n-triples
<http://example.org/#john> <http://example.org/#name> "John Smith" .
```

The layout can be used to go from the RDF form to the structured value form by
**serialization**.
It can also be used in the opposite direction from the structured value form to
the pure RDF form by **deserialization**.