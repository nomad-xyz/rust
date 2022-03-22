## Nomad Config file

This is a crate for working with nomad configuration files. These config files
contain information about the state of Nomad deployments.

It also includes an auto-generated TS/WASM library.

### Design Notes

The core library is mostly a JSON config file format. We define Rust structs
and TS types for all parts of this config.

In TS, the object is a native JS object. It is _not_ a reference to a wasm type.
Assignment and access can be done as normal. However, we have also exported
functions that perform consistency-critical operations like `addNetwork` and
`addCore`. We strongly recommend using these instead of assigning to the
relevant sections.

### Usage 

#### Typescript 

```typescript
import * as configuration from "@nomad-xyz/configuration"

const config = configuration.getBuiltin("production")

console.log(`Environment: ${config.environment}`)
```

#### Rust 

// TODO 

### Building

- `$ cargo build`

To build the wasm library:

- [Install wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- `$ ./package-it.sh`

`wasm-pack` docs are found [here](https://rustwasm.github.io/wasm-pack/book/).

### Testing

- `$ cargo test`

### Documenting

- `$ cargo docs --open`

### Publishing

- `$ cargo publish`
- `$ ./publish_it.sh`

### Development note

To work around some `wasm-bindgen` limitations, we currently (unfortunately)
have to manually define TS types for the rust structs. These are found in the
`data` directory. When a rust struct is updated or added, the corresponding
definitions should be added in `data/definitions.ts` and `data/types.rs`. At
compile-time these files are combind to `src/wasm/types.rs`.

In the future it'd be cool to auto-generate this code :)
