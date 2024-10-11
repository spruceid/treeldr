# Types

The data-model presented so far in this chapter is fundamentally *untyped*.
However, it can (and will in the next chapter) be useful to formally describe
a subset of values sharing a given *shape*.
In this section we define types as sets of tree values.

There are three primary sorts of types:
  - Data-types, describing sets of literal values;
  - Record types: describing sets of record values;
  - List types: describing sets of list values.

In addition, it is possible to compose new types by union or intersection.

## Syntax

Just like for the data-model itself, we define a syntax for types.

```abnf
type = datatype | record-type | list-type
type-ref = ALPHA *(ALPHA | DIGIT)
type-expr = type-ref | type
```

### Type references and expressions

A type reference, corresponding to the `type-ref` production in the above
grammar, is a name referring to a type definition.
A type expression (`type-expr` production) is either a type reference or
definition.

## Datatype

The following core type references are always defined:
  - `unit`
  - `boolean`,
  - `number`,
  - `bytes` (byte string),
  - `string` (text string)

## Record

### Syntax

```abnf
record-type   = "{" [ binding-types ] ws "}"
binding-types = ws binding-type | ws binding-type ws ":" ws ","
binding-type  = key ws ":" ws type-expr
```

For example:

```ts
{
	"id": string,
	"name": string
}
```

## List

### Syntax

```abnf
list-type = "[" ws type-expr ws "]"
```