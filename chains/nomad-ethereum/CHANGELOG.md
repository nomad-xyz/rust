# Changelog

### Unreleased

- Add `EthereumError` error enum to wrap ethers and gelato errors (ethereum-specific)
- Make existing contract and indexer methods return `Result<_, EthereumError>` now instead of using old `nomad_core::ChainCommunicationError`
- impl `std::fmt::Display` for `EthereumHome` and `EthereumReplica`
- use gelato-sdk as a github dep rather than a crate
- adds a changelog
