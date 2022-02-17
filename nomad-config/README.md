## Nomad Config file

This is a crate for working with nomad configuration files. These config files
contain information about the state of Nomad deployments.

### Building

- `$ cargo build`

To build the wasm library:

- [Install wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- `$ wasm-pack build --target nodejs --scope nomad-xyz`

`wasm-pack` docs are found [here](https://rustwasm.github.io/wasm-pack/book/).
