<img src=".github/images/Logo-White.svg" alt="Nomad logo" style="width: 100%; background: black;"/>

## Nomad

Nomad is a cross-chain communication standard that supports passing messages between blockchains easily and inexpensively. Like [IBC](https://ibcprotocol.org) light clients and similar systems, Nomad establishes message-passing channels between chains. Once a channel is established, any application on that chain can use it to send messages to others chains.

Nomad is an implementation and extension of the [Optics protocol](https://medium.com/celoorg/announcing-optics-a-gas-efficient-interoperability-standard-for-cross-chain-communication-e597163b2) (**OPT**imistic **I**nterchain **C**ommunication), originally developed at Celo.

Compared to light clients, Nomad has weaker security guarantees and a longer latency period. However, these tradeoffs allow Nomad to be implemented on any smart contract chain without expensive light client development. Unlike light clients, Nomad does not use gas verifying remote chain block headers.

Nomad is designed to prioritize:

- Operating costs: No gas-intensive header verification or state management
- Implementation speed and cost: Uses simple smart contracts without complex cryptography
- Ease of use: Straightforward interface for maintaining xApp connections
- Security: Relies on a 1/n honest watcher assumption for security

You can read more about Nomad's architecture at our [main documentation site](https://docs.nomad.xyz).

## Nomad Rust Repository

Nomad's off-chain systems are written in Rust for speed, safety and reliability. (Nomad's on-chain systems are written in Solidity and are available [here](https://github.com/nomad-xyz/monorepo).)

### Rust Setup

- Install `rustup` from [here](https://rustup.rs/) and run it

Note: You should be running at least version `1.52.1` of the rustc compiler. Check it with `rustup --version`

```
$ rustup --version
rustup 1.24.2 (755e2b07e 2021-05-12)
info: This is the version for the rustup toolchain manager, not the rustc compiler.
info: The currently active `rustc` version is `rustc 1.52.1 (9bc8c42bb 2021-05-09)`
```

Rust uses `cargo` for package management, building, testing and other essential tasks.

For Ethereum and Celo connections we use
[ethers-rs](https://github.com/gakonst/ethers-rs). Please see the docs
[here](https://docs.rs/ethers/0.2.0/ethers/).

Nomad uses the tokio async runtime environment. Please see the docs
[here](https://docs.rs/tokio/1.1.0/tokio/).

### Running the Test Suite

- `cargo test --workspace --all-features`

This will run the full suite of tests for this repository.

### Generate Documentation

- `cargo doc --open`

This will generate this repos documentation and open it in a web browser.

### Agent Architecture

The off-chain portion of Nomad is a set of agents each with a specific role:

- `updater`
  - Signs update attestations and submits them to the home chain
- `watcher`
  - Observes the home chain
  - Observes one or more replica chains
  - Check for fraud
  - Submits fraud to the home chain
  - If configured, issues emergency stop transactions
- `relayer`
  - Relays signed updates from the home chain to the replicas
- `processor`
  - Retrieves Merkle leaves from home chain
  - Observes one or more replica chains
  - Generates proofs for passed messages
  - Submits messages with proofs to replica chains

### Repository Layout

- `nomad-base`
  - A VM-agnostic toolkit for building agents
    - Common agent structs
    - Agent traits
    - Agent settings
    - NomadDB (RocksDB)
    - Concrete contract objects (for calling contracts)
    - VM-agnostic contract sync
    - Common metrics
- `nomad-core`
  - Contains implementations of core primitives
    - Core primitives
    - Core data types
    - Contract and chain traits
- `nomad-types`
  - Common types used throughout the stack
- `chains`
  - A collection of crates for interacting with different VMs
    - Ethereum
    - More coming...
- `accumulator`
  - Contains Merkle tree implementations
- `agents`
  - A collection of VM-agnostic agent implementations
- `configuration`
  - An interface for persisting and accessing config data
    - JSON config files (for development, staging, production)
    - An interface for retrieving agent and system config
    - An interface for retrieving agent secrets

## Contributing

All contributions, big and small, are welcome. All contributions require signature verification and contributions that touch code will have to pass linting and formatting checks as well as tests.

### Commit signature verification

Commits (and tags) for this repository require signature verification. You can learn about signing commits [here](https://docs.github.com/en/enterprise-server@3.3/authentication/managing-commit-signature-verification/signing-commits).

After signing is set up, commits can be signed with the `-S` flag.

- `git commit -S -m "your commit message"`

### Testing, Linting and Formatting

If your commits have changed code, please ensure the following have been run and pass before submitting a PR:

```
cargo check --workspace
cargo test --workspace --all-features
cargo fmt --all
cargo clippy --workspace --all-features -- -D warnings
```

## Release Process

### Overview

We make releases within the `rust` repository specific to the crate(s) that will be consumed (e.g. agents@1.0.0, configuration@1.0.0, accumulator@1.0.0, etc).

We follow [Semantic Versioning](https://semver.org/), where breaking changes constitute changes that break agent configuration compatibility.

Releases are managed on GitHub [here](https://github.com/nomad-xyz/rust/releases).

### Aggregating Release Notes

- Want to aggregate list of all changes since last release
- Run `git diff <sha of last release> HEAD -- **CHANGELOG.md`
- Manually consolidate diffs into a single list for release notes

### Bumping Versions

- Bump package versions in all relevant `Cargo.toml` files
- Bump the package versions in all relevant `CHANGELOG.md` files
- E.g. for an `agents` release, this would entail bumping all agents in `rust/agents`
- Make/merge a PR declaring the new version you are releasing (e.g. "Bumping agents for release agents@1.0.1")

### Making a New Release

- Visit the [releases page](https://github.com/nomad-xyz/rust/releases) for the `rust` repo
- Draft a new release using the name of the release as the title and tag (e.g. agents@1.0.1)
- Include your compiled list of release notes
- Publish release

## Advanced Usage

### Building Agent Images

There exists a docker build for the agent binaries. These docker images are used for deploying the agents in a production environment.

```
./build.sh <image_tag>
./release.sh <image_tag>
```

### Adding a New Agent

- Run `cargo new $AGENT_NAME`
- Add the new directory name to the workspace `Cargo.toml`
- Add dependencies to the new directory's `Cargo.toml`
  - Copy most of the dependencies from `nomad-base`
- Create a new module in `src/$AGENT_NAME.rs`
  - Add a new struct
  - Implement `nomad_base::NomadAgent` for your struct
  - Your `run` function is the business logic of your agent
- Create a new settings module `src/settings.rs`
  - Reuse the `Settings` objects from `nomad_base::settings`
  - Add your own new settings
  - Make sure to read the docs :)
- In `$AGENT_NAME/src/main.rs`:
  - Add `mod` declarations for your agent and settings modules
  - Create `main` and `setup` functions
  - Use the implementation in `agents/kathy/src/main.rs` as a guide
- Add required config to `configuration/configs/*` for the agent

## Miscellaneous

### Useful `cargo` Extensions

- `tree`
  - Show the dependency tree. Allows searching for specific packages
  - Install: `cargo install cargo-tree`
  - Invoke: `cargo tree`
- `clippy`
  - Search the codebase for a large number of lints and bad patterns
  - Install: `rustup component add clippy`
  - Invoke: `cargo clippy`
- `expand`
  - Expand macros and procedural macros. Show the code generated by the
    preprocessor
  - Useful for debugging `#[macros]` and `macros!()`
  - Install: `cargo install cargo-expand`
  - Invoke `cargo expand path::to::module`
