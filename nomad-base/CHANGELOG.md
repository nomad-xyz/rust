# Changelog

- fix: instrument futures, not joinhandles

### Unreleased

- Have both Home/Replica and Home/Replica indexers return `Self::Error`
- Add `Home` and `HomeIndexer` support for Substrate variants as well as allowing configuration of Substrate objects
- Add `AttestationSigner` type alias that wraps `EthereumSigner`
- Add new `ChainCommunicationError` to wrap `nomad_ethereum::EthereumError` and `nomad_substrate::SubstrateError`
- Add implementations for converting chain-specific error enums into `ChainCommunicationError` to catch reverts
- un-nest, simplify & add event to setup code for determining which replicas to
  run
- un-nest, simplify & add event to setup code for config source discovery
- emit event for source of config loaded at bootup
- implement `std::fmt::Display` for `Home` and `Replica` enums
- add home and remote labels to contract sync metrics for event differentiation
- add `CONFIG_URL` check to `decl_settings` to optionally fetch config from a remote url
- prometheus metrics accepts port by env var
- bug: add checks for empty replica name arrays in `NomadAgent::run_many` and
  `NomadAgent::run_all`
- add `previously_attempted` to the DB schema
- remove `enabled` flag from agents project-wide
- adds a changelog
