# Changelog

### Unreleased

- Remove `Signers` enum in favor of breaking into separate `EthereumSigners` and `SubstrateSigners` types for submitting txs
- Remove `ChainCommunication` in favor of new `ChainCommunicationError` error wrapper in `nomad-base`
- Have `Home`, `Common`, and `ConnectionManager` traits return associated type errors instead of legacy `nomad_core::ChainCommunicationError`
- require `Common: std::fmt::Display`
- refactor: Add IRSA credentials to client instantiation
- implement `Encode` and `Decode` for `bool`
- adds a changelog
