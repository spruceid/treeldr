# RDF Basics

The Resource Description Framework (RDF) is a very simple but powerful data-model designed for the Semantic Web.
In this model, every piece of data is a node in a labeled directed graph. Each
node is called a **Resource**, and resources are connected together using
*properties*, which are resources themselves.

TODO example

## Lexical Representations

Resources are given *lexical representations* that uniquely identify them across
the Web and give them meaning.
There are three kind of lexical representations a resource can have:
  - International Resource Identifiers (IRI), similar to URLs but with
    international characters;
	Example: `https://example.org/Ῥόδος`
  - Literal values (a text string, a number, etc.);
  - Blank node identifiers, that locally identifies a resource in a give RDF
    document. Blank node identifiers do not cross document boundaries and are
    solely used to give a temporary name to nameless resources.
	Such identifiers are similar to IRIs with `_` as scheme.
	Example: `_:SomeAnonymousResource`.
  
This lexical representation allows the definition of some
proper textual syntaxes for RDF, such as N-Triples or RDF-Turtle. The former is
a simple enumeration of the graph's edges, called triples.
A triple has three parameters: the subject resource, the property, and the
object (value) of the property.

TODO example

## Interpretations

The mapping from lexical representation to resource is called an
**interpretation**. It is a partial function mapping the set of lexical
representations to the set of resources.

TODO illustration

As shown above, a resource may have more than one lexical representation.
In this case the shape of the lexical representation of a graph may differ from
its actual shape.
In the following graph, `_:A` and `_:B` have the same
interpretation (by definition of `http://www.w3.org/2002/07/owl#sameAs`).
As a result, its lexical form (on the left) contains two nodes, while its
interpreted form (on the right) contains only a single node.

TODO example

## Datasets

An RDF dataset is a collection of RDF graphs. A graph can be either the *default
graph*, or a *named graph*. A named graph is also a resource.

TODO example

The N-Quads syntax can be used to represent RDF datasets.
It is similar to the N-Triples syntax showed previously above, but lists *quads*
instead of *triple*, where an optional fourth parameter is here to specify in
which named graph the triple occurs. If no fourth parameter is specified, it
means the triple appears in the default graph.

TODO example