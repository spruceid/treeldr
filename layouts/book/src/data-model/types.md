# Types

A type is a set of values.

## Syntax

```abnf
type = record-type | list-type | value
```

## Type reference

A type reference is a name referring to a type definition.

```abnf
type-ref = ALPHA *(ALPHA | DIGIT)
```

## Type expression

A type expression is either a type reference or definition.

```abnf
type-expr = type-ref | type
```

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

```
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