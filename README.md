# wasm-chisel

Some useful utilities to transform WebAssembly binaries, most importantly for WebAssembly used in a deterministic / blockchain context,
such as with [ewasm].

## Library

### remapimports

Provide a list of imports (with namespace and name) and replace them with a new set of namespace and name pairs.

This can be very useful together with compilers, which do not support the specification of a namespace in imports yet. As of writing mid-2018,
that includes pretty much every compiler (one exception is AssemblyScript).

### trimexports

Removes all exports, but the ones specified.

This comes with some presets:
- `ewasm`: keeps `main` and exported memory
- `pwasm`: keeps `_call`

### verifyimports

Verifies that the module's imports are compliant with the provided import interface.
Can be set to require the existence of the entire import set, or just the validity of existing imports with matching identifiers.
Can be set to allow or prohibit unlisted additional imports.

The following presets are provided:
- `ewasm`: Verifies the ewasm [EEI](https://github.com/ewasm/design/blob/master/eth_interface.md). Disallows unlisted imports, and does not require that the entire interface be imported.

### verifyexports

Verifies that the module's exports are compliant with the provided export interface.
Can be set to allow or prohibit unlisted additional exports.

The following presets are provided:
- `ewasm`: Verifies that the `main` function and `memory` is exported. Disallows any unlisted exports.

### deployer

Wraps module into an ewasm-compatible constructor. It has two presets:
- `memory`: wrap the module as a pre-defined memory section
- `customsection`: include the module as a custom section

## CLI

`chisel` is available as a command line tool.

It uses features implemented in the library as well in [wasm-gc] and [wasm-utils]. It comes with a configuration file `chisel.yml`.

`chisel run`: searches for `chisel.yml` in the current directory, if not specified otherwise using the flag `-c`. Runs the modules specified in the configuration, outputs a new file if any changes were made by translator or creator modules, and prints a brief report of each module's results.

## Configuration file

The configuration file starts with a ruleset entry, where the name can be anything. Inside the ruleset are its options.

The only required field is `file`, which specifies the path to the Wasm binary to be chiseled.

Optionally, one may also specified an output file through the `output` option.

It is important to note that the configuration parsing will not work if all the rules are prepended with a hyphen. Please avoid this until the configuration parser is generalized.

```yaml
ewasm:
  file: "target/wasm32-unknown-unknown/release/sentinel.wasm"
  output: "out.wasm"
  remapimports:
    preset: "ewasm"
```

## sentinel.rs

TBA

[ewasm]: http://github.com/ewasm
[wasm-gc]: https://github.com/alexcrichton/wasm-gc
[wasm-utils]: https://github.com/paritytech/wasm-utils
