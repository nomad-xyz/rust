# Changelog

### Unreleased

- Remove `ChainCommunication` in favor of new `ChainCommunicationError` in `nomad-base`
- Have `Home`, `Common`, and `ConnectionManager` traits return associated type errors instead of legacy `nomad_core::ChainCommunicationError`
- require `Common: std::fmt::Display`
- refactor: Add IRSA credentials to client instantiation
- implement `Encode` and `Decode` for `bool`
- adds a changelog
