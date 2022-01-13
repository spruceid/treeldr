# TreeLDR

An intuitive Linked Data schema definition language and boilerplate code generator.
TreeLDR can be used to produce JSON Schemas, JSON-LD contexts, migration strategies, blockchain publishing routines, etc.
and entire SDKs in various target programing languages such as Python, Java and more.
This way, developers can define data structures in a familiar way and focus purely on the application level.

Ideally, TreeLDR supports more than just JSON-based data formats, such as CBOR, capnproto, protobuf, ASN.1, and more.

## Usage

TreeLDR strawman example:
```
use schema_org::URL;
use imsglobal::{Credential, Department, Degree, Year, Honors};

Diploma :: Credential {
    department :: Department required
    degree :: Degree required
    year :: Year required
    honors :: Honors required
    url :: URL required
}

Profile{signing} = // ...
Profile{upgrade} = // ...
Profile{downgrade} = // ...
Profile{issuers} = // ...
```

### Generating a Python package

```bash
$ treeldr ./diploma ./openbadges3
$ ./openbadges3/setup.py
```

Use of a Python package to work with college diplomas, generated from TreeLDR
that supports JSON-LD:
```python
import openbadges3 as ob;

diploma = ob.Diploma(
    department=ob.Department("engineering"),
    graduation_year=ob.Year("2022"),
    degree=ob.Degree("Bachelor of Science"),
    honors=ob.Honors("Cum Laude"),
)
signed_diploma = diploma.sign(env.SIGNING_KEY)

# Print a W3C Verifiable Credential
print(signed_diploma.as_vc())
```

Python is just one of many languages that TreeLDR should support, which include
also JavaScript, Java, Scala, Ruby, Go, PHP, and more.

## Features to consider
- The ability to choose verification methods and valid key material types,
  perhaps using [cryptoscript](https://github.com/spruceid/cryptoscript).
  DIDKit could be embedded in package outputs to provide integrated
  functionality.
- The ability to specify a trust framework, such as the trusted issuers,
  presenters, holders, etc. and their roles.
- Data structure composability, such as using sumtypes and GADTs to deduplicate
  the amount of scaffolding and data field repetition.
- Deep interoperability with Linked Data.
- Specification of migration strategies as data models evolve, for ease of
  upgrades or rollbacks. This could be combined with semantic operations such
  as `fullName = firstName + middleName + lastName` if adding a `fullName`
  field to the data model from constituent fields.
- Automatic publishing of schemas, packages, revocation lists, accumulator
  values, and trusted issuer lists to a [verifiable data
  registry](https://www.w3.org/TR/vc-data-model/#dfn-verifiable-data-registries).
