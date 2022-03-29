# TypeScript package generator

This library provides a TypeScript code generator for TreeLDR.
It provides two subcommands to the main command line interface:
`typescript` to generate a TypeScript file and `typescript-package` to
generate a TypeScript package.

### Generate a TypeScript file

The following command will produce a TypeScript file to the standard output:
```txt
treeldr -i input.tldr typescript
```

A class is generated for each layout defined in the inputs.

### Generate a TypeScript package

The following command will produce a TypeScript package.
```txt
treeldr -i input.tldr typescript-package package-name
```

The `package-name` parameter is the only required value.
By default the package is generated in the current working directory.
You can specify an other directory using the `-d`/`--dir` option.

The package generator is very conservative: any modification to the
generated files outside of the `src/main.ts` file will be preserve if you
try to regenerate the package.
