# Values

TreeLDR supports several types of values that can be constructed using specific syntax.

## RDF Resource

An RDF Resource represents a resource identified by a URI or an RDF literal value.

### URI Resource

```tldr
<https://example.org/someResource>
```

### RDF Literal

RDF literals represent typed or untyped string values. They are enclosed in single quotes and optionally followed by a type URI.

- **Typed literal:**  
  A literal with a specified type is written between single quotes and a type
  suffix preceded by `^^` (borrowed from the N-quads syntax):

  ```tldr
  'example'^^<http://www.w3.org/2001/XMLSchema#string>
  '42'^^<http://www.w3.org/2001/XMLSchema#integer>
  ```

- **Untyped literal:**  
  When the type is `http://www.w3.org/2001/XMLSchema#string`, it can be omitted for convenience:

  ```tldr
  'example'
  ```

## Literal

Literals represent basic data types.

### Null

A null literal represents the absence of a value.

```tldr
null
```

### Boolean

Boolean literals represent logical truth values.

```tldr
true
false
```

### Number

Number literals represent numerical values, including integers and floating-point numbers.

```tldr
1
3.14
-42
```

### Text String

Text strings are sequences of Unicode characters enclosed in double quotes.

```tldr
"foo"
"Hello, world!"
```

### Byte String

Byte strings represent sequences of raw bytes, but the syntax for this is not yet defined.

## List

Lists are ordered collections of values. Elements can be of any type.

```tldr
[1, 2, 3, "string", true, null]
```

## Map

Maps are collections of key-value pairs, where keys can be of any literal type and values can be any value.

```tldr
{
	"key" = "value",
	0 = null,
	true = [1]
}
```
