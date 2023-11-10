# Values

This section specifies all the *structured values* that can be processed and/or produced using TreeLDR layouts. A value can be either:
  - a **literal** value, representing any atomic value;
  - a **record**, representing a collection of key-value pairs;
  - a **list**, representing a sequence of values.

This data-model is close to the JSON data model, with some notable exceptions:
  - The value space of numbers is all the rational numbers, and not just decimal numbers;
  - Surrogate Unicode code points are not allowed in the lexical representation of text strings;
  - There is a dedicated datatype for binary strings.

## Syntax

```abnf
value = literal | record | list
```

## Literals

A literal value can be:
  - the unit value **unit** written `()`
  - a **boolean** value, either `true` or `false`,
  - a **number**, written as a decimal number (e.g. `12.9`),
  - a **binary string** written as an hexadecimal value preceded by a `#` character,
  - a **text string**, written between double quotes `"`.

### Syntax

```abnf
literal = unit | boolean | number | bytes | string
```

## Unit

Unit is a singleton datatype.

### Syntax

The unit value is written using a pair of parentheses `()`.

```abnf
unit = "()"
```

## Boolean

The boolean datatype contains the two values `true` and `false`.

### Syntax

```abnf
boolean = "true" | "false"
```

## Number

The number datatype contains all the [rational numbers (ℚ)](https://en.wikipedia.org/wiki/Rational_number).

### Syntax

```abnf
number = decimal | fraction
decimal = +DIGIT [ "." DIGIT ]
fraction = +DIGIT \ NZDIGIT +DIGIT

NZDIGIT = "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
```

## Binary String

The binary string datatype contains any string of bytes.

### Syntax

```abnf
bytes = "#" *( HEXDIGIT HEXDIGIT )
```

## Text String

The text string datatype contains any string of [Unicode scalar value](https://www.unicode.org/glossary/#unicode_scalar_value), which is any [Unicode code point](https://www.unicode.org/glossary/#code_point) other than a [surrogate code point](https://www.unicode.org/glossary/#surrogate_code_point).

### Syntax

A string is written as a sequence of characters between double quotes. Any
Unicode scalar value is allowed starting from U+0020 inclusive (the whitespace
character) except for U+0022 (the quotation mark) and U+005C (the reverse
solidus) which must be escaped along with control characters before U+0020.

```abnf
string = quotation-mark *char quotation-mark

char = unescaped
     / escape (
        %x22 /                 ; "    quotation mark  U+0022
        %x5C /                 ; \    reverse solidus U+005C
        %x2F /                 ; /    solidus         U+002F
        %x62 /                 ; b    backspace       U+0008
        %x66 /                 ; f    form feed       U+000C
        %x6E /                 ; n    line feed       U+000A
        %x72 /                 ; r    carriage return U+000D
        %x74 /                 ; t    tab             U+0009
        %x75 "{" 1*6HEXDIG "}" ; u{XXXX}              U+XXXX
     )

escape = %x5C              ; \
quotation-mark = %x22      ; "
unescaped = %x20-21 / %x23-5B / %x5D-D7FF / %xE000-10FFFF
```

## Records

The record datatype contains all finite [partial functions](https://en.wikipedia.org/wiki/Partial_function) from keys to values, where keys are text string literals.

### Syntax

```abnf
record = "{" [bindings] ws "}"
bindings = ws binding | ws binding ws "," ws bindings
binding = key ws ":" ws value
key = string
ws = *WS
```

## Lists

The list datatype contains all the finite sequences of values.

### Syntax

```abnf
list-type = "[" [items] ws "]"
items = ws value | ws value ws "," ws items
```