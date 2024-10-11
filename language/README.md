# TreeLDR Schema Definition Language

The particularity of TreeLDR expressions is that they must be *reversible*.

# Lists

List type:
```tldr
[T]
```

# Tuples

Tuple type:
```tldr
(T1, ..., TN)
```

# Set

Set type:
```tldr
{ T }
```

Set expression:
```tldr
set { value1, ..., valueN } as ...
```

Set selector expression:
```tldr
all ?t where { ... } as ...
```

# Optional type

Option type:
```tldr
T?
```

Which is syntactic sugar for:
```tldr
(option T)
```

Option expression:
```tldr
(some value)
none
```

Optional selector expression:
```tldr
optional ?t where { ... } as ...
```

# Struct

Struct type:
```tldr
struct ?self, ?value where { ?self rdf:type ex:Foo . } {
	field1: type1 = value1,
	fieldN: typeN = valueN
}
```

Constructing a struct:
```tldr
{
	field1: value1, .., fieldN: valueN
}
```

# Function

Function type:
```tldr
fn (T1, ..., TN) -> T
```

Function arguments can be tagged with the `const` marker, meaning that its value
must be known at compile time.
This is required to make functions *invertible*.

Calling a function:
```tldr
(id a1 ... aN)
```

## Built-in functions

- `iri: fn (resource) -> iri`: Returns the IRI of a `resource`.
- `iri_to_string: fn (iri) -> string`: Returns the `string` representation of an IRI.
- `bid: fn (resource) -> bid`: Returns the blank node identifier of a `resource`.
- `bid_to_string: fn (bid) -> string`: Returns the `string` representation of a blank node identifier.
- `term: fn (resource) -> term`: Returns the `term` representation of a `resource`.
- `strip_prefix: fn (const string, string)`: Strip the prefix of a string.
- `strip_suffix: fn (string, const string)`: Strip the suffix of a string.
- `string_to_u32: fn (string) -> u32`: Parse a `string` and returns a `u32`.

# Selectors

Optional:
```tldr
optional ?t where { ... } as ...
```

Required:
```tldr
required ?t where { ... } as ...
```

Arbitrary:
```tldr
all ?t min 0 max 1 where { ... } as ...
```

# Definition

```tldr
Foo = Type
```

```tldr
(Foo T1 ... TN) = Type
```