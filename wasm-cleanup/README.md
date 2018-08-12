# wasm-cleanup

This tool transform WebAssembly S-expressions (WAST) code to an output conforming to the [*eWASM Contract Interface*](https://github.com/ethereum/evm2.0-design).

It is especially useful for using together `clang` to create eWASM contracts, because `clang` and C has a few limitations regarding WebAssembly currently.

## Transformations

### Exports

`clang` will export every global method, which is expected. For some reason making methods `static` or `inline` will not produce the proper results.

`ewasm-cleanup` will remove every export bar `main` and `memory`.

### Imports

#### Ethereum module

Most of the current toolchains (clang for C and mir2wasm for Rust) do not support WebAssembly's two-level imports (namespace and method name). They mostly
hardcode `env` as the namespace.

`ewasm-cleanup` will transform these to imports matching the *Ethereum Environment Interface*.

#### libc methods

`clang` considers `memset` and `memcpy` special. It even detects patterns of them and replaces it with the internal `memcpy` and `memset`. Unfortunately
it isn't fully supported in the WebAssembly target and it ends up with `memset` and `memcpy` being an import.

`ewasm-cleanup` will insert an implementation of `memset` and `memcpy`.

## Author

Alex Beregszaszi

## License

MIT
