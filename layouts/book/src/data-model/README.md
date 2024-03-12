# Data Model

Layouts define a mapping from RDF datasets to tree data.
The RDF dataset model is already specified by the RDF specification.

This section specifies all the *structured values* that can be processed and/or produced using TreeLDR layouts. A value can be either:
  - a **literal** value, representing any atomic value;
  - a **record**, representing a collection of key-value pairs;
  - a **list**, representing a sequence of values.

This data-model is close to the JSON data model, with some notable exceptions:
  - The value space of numbers is all the rational numbers, and not just decimal numbers;
  - Surrogate Unicode code points are not allowed in the lexical representation of text strings;
  - There is a dedicated datatype for binary strings.

This chapter details the data model for tree values.

# Syntax

In addition, to write the formal specification of layouts, we also define
a syntax for values, along with a type system.

```abnf
value = literal | record | list
```