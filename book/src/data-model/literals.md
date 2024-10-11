# Literals

A literal value can be:
  - the unit value **unit** written `()`
  - a **boolean** value, either `true` or `false`,
  - a **number**, written as a decimal number (e.g. `12.9`),
  - a **binary string** written as an hexadecimal value preceded by a `#` character,
  - a **text string**, written between double quotes `"`.

## Syntax

The ABNF grammar of literals is as follows:

```abnf
literal = unit | boolean | number | bytes | string
```

# Unit

Unit is a singleton datatype. It contains one unique value, the unit value.
This value is very similar to JSON's `null` value.

## Syntax

The unit value is written using a pair of parentheses `()`.

```abnf
unit = "()"
```

# Boolean

The boolean datatype contains the two values `true` and `false`.

## Syntax

The ABNF grammar of boolean values is as follows:

```abnf
boolean = "true" | "false"
```

# Number

The number datatype contains all the [rational numbers (ℚ)](https://en.wikipedia.org/wiki/Rational_number).

## Syntax

Numbers are written either as decimal numbers, or as fractions of two integer
numbers.

```abnf
number = decimal | fraction
decimal = +DIGIT [ "." DIGIT ]
fraction = +DIGIT "\" NZDIGIT +DIGIT

NZDIGIT = "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
```

# Binary String

The binary string datatype contains any string of bytes.

## Syntax

```abnf
bytes = "#" *( HEXDIGIT HEXDIGIT )
```

# Text String

The text string datatype contains any string of [Unicode scalar value](https://www.unicode.org/glossary/#unicode_scalar_value), which is any [Unicode code point](https://www.unicode.org/glossary/#code_point) other than a [surrogate code point](https://www.unicode.org/glossary/#surrogate_code_point).

## Syntax

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