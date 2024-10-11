# Lists

The list datatype contains all the finite sequences of values.

## Syntax

```abnf
list-type = "[" [items] ws "]"
items = ws value | ws value ws "," ws items
```