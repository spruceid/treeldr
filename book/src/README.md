# TreeLDR

TreeLDR is a functional data transformation language based on the Resource Description Framework (RDF). It can be used to serialize RDF datasets into algebraic data values, and back. What makes TreeLDR unique is that functions are **always reversible**: writing a serialization function gives the deserialization function for free.

## Basic function

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

## RDF as an intermediate representation

Because every functions are reversible, it is actually possible to use TreeLDR without never inputting or outputting any RDF dataset whatsoever. Instead RDF can be hidden away as an intermediate representation to a data transformation operation.

For instance, consider the following new function:
```tldr
let (bar input) = {
	"identifier": (id_of input),
	"data": {
		"fullName": (string_of (get schema:name input)),
		"dateOfBirth": (date_of (get schema:birthDate input))
	}
}
```
This function is very similar to our original `foo`. It uses the same expressions to populate the map from the `input` RDF resource (the *semantics* is the same), but the entry names and overall structure is different.

We can combine the two function to translate any value from one structure to the other:
```tldr
let (foo_to_bar json_input) =
	let (foo rdf_resource) = json_input;
	(bar rdf_resource)
```
Here we first call the inverse of `foo` on the `json_input`, creating an intermediate `rdf_resource`. This is done using the `let (f x) = y` syntax that finds the input `x` such that `(f x)` is `y`. Then we call `bar` on that intermediate value.

Let's bring back the output of our previous example:
```json
{
	"id": "https://example.org/johnSmith",
	"name": "John Smith",
	"birth_date": "2024-12-03"
}
```
If we call the `foo_to_bar` function on this value, we will get the following output:
```json
{
	"identifier": "https://example.org/johnSmith",
	"data": {
		"fullName": "John Smith",
		"dateOfBirth": "2024-12-03"
	}
}
```
We got this output without seeing any RDF dataset. And it is still possible to reverse the `foo_to_bar` function to convert the output back.