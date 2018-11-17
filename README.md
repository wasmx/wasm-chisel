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

### verifyimports (WIP)

TBA

### verifyexports

Validates that the module's exports are compliant with the given export interface.
Can be set to allow or prohibit additional exports that are not listed.

The following presets are provided:
- `ewasm`: Verifies that the `main` function and `memory` is exported. Disallows any unlisted exports.

### deployer

TBA

## CLI (WIP)

`wasm-chisel` is available as a command line tool.

It uses features implemented in the library as well in [wasm-gc] and [wasm-utils]. It comes with a configuration file `chisel.yaml`.

## Configuration file (WIP)

The configuration file starts with a ruleset entry, where the name can be anything. Inside the ruleset are its options.

```yaml
ewasm:
  - file: "target/wasm32-unknown-unknown/release/sentinel.wasm"
  - remapimports:
    - style: ewasm
  - deployer
```

## sentinel.rs

TBA

[ewasm]: http://github.com/ewasm
[wasm-gc]: https://github.com/alexcrichton/wasm-gc
[wasm-utils]: https://github.com/paritytech/wasm-utils
