# Records

The record datatype contains all finite [partial functions](https://en.wikipedia.org/wiki/Partial_function) from keys to values, where keys are text string literals.

## Syntax

```abnf
record = "{" [bindings] ws "}"
bindings = ws binding | ws binding ws "," ws bindings
binding = key ws ":" ws value
key = string
ws = *WS
```