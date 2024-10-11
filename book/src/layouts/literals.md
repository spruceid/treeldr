# Literal Layouts

A literal layout matches any literal tree value and any RDF literal lexical
representation satisfying a given set of constraints.
Literal layouts are refined further into five categories corresponding to the
five primitive data-types defined by TreeLDR's data-model (unit, boolean,
number, binary string and text string).
The following table summarizes what matches a literal layout in the tree space,
and RDF space.

| Literal layout type | Tree space | RDF space |
| ------------------- | ---------- | --------- |
| Unit    | Unit, or any predefined constant | Any resource |
| Boolean | Any boolean | Any resource with a literal representation of type <http://www.w3.org/2001/XMLSchema#boolean> |
| Number | Any number | Any resource with a literal representation of type <http://www.w3.org/2001/XMLSchema#decimal> |
| Binary string | Any binary string | Any resource with a literal representation of type <http://www.w3.org/2001/XMLSchema#base64Binary> or <http://www.w3.org/2001/XMLSchema#hexBinary> |
| Text string | Any text string | Any resource with a literal representation |

Literal layouts are represented by values of the following type:

```ts
type LiteralLayout =
	  UnitLayout
	| BooleanLayout
	| NumberLayout
	| BinaryStringLayout
	| TextStringLayout
```

## Unit

The unit layout matches, in the tree space, the unit value (or any given
constant) and in the RDF space, any resource.

Unit layouts are represented by values of the following type:

```ts
type UnitLayout = LayoutDefinition & {
	"type": "unit",
	"const"?: Any
}
```

The optional `const` attribute specifies which tree value matches the layout.
The default value for the `const` attribute is the unit value `()`.

## Boolean

```ts
type BooleanLayout = LayoutDefinition & {
	"type": "boolean",
	"resource": Resource
}
```

## Number

```ts
type NumberLayout = LayoutDefinition & {
	"type": "number",
	"resource": Resource
}
```

## Binary String

```ts
type BinaryStringLayout = LayoutDefinition & {
	"type": "bytes",
	"resource": Resource
}
```

## Text String

```ts
type TextStringLayout = LayoutDefinition & {
	"type": "string",
	"resource": Resource,
	"pattern"?: Regex
}
```