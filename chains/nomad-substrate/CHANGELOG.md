# Changelog

### Unreleased

- Change `double_update` to a NOP
- Update `update` method with new max index field
- `produce_update` checks that tree has at least 1 element (bug fix)
- Add timelag functionality to `NomadOnlineClient` which wraps storage fetches with timelagged fetches
- Add methods to macros.rs to allow for configuring Substrate objects from configuration conf objects
- Add initial `nomad-substrate` home implementation and replica/xapp stubs
- Return `SubstrateError` for fetches/calls as well as indexing
- Add `SubstrateError` type which wraps all subxt and scale-value deserialization errors (substrate-specific)
