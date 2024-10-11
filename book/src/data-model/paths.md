# Path

A **path** is a sequence of *segments* leading to a unique node value in a tree.
Each segment correspond to a tree branch, such as a field name or list index.

For instance, consider the following tree value:
```json
{
  "foo": [
    { "bar": 1 },
    { "bar": 2 }
  ]
}
```

The following path leads to the value `2`:
```
foo/1/bar
```

## Path validation

TreeLDR layouts can be seen as type definitions for trees.
We can use the layout definition to validate a path before using it to access
an actual tree value.

For instance, consider the following layout for the above value:
```json
{
  "type": "record",
  "fields": {
    "foo": {
      "value": {
        "type": "list",
        "node": {
          "value": {
            "type": "record",
            "fields": {
              "bar": {
                "value": { "type": "number" }
              }
            }
          }
        }
      }
    }
  }
}
```

This layout defines the following path family (a regular expression):
```
(foo(/[0-9]+(/bar)?)?)?
```