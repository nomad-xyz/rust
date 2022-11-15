# Changelog

### Unreleased

- use `std::fmt::Display` to log contracts
- fix: instrument futures, not joinhandles

### agents@1.1.0

- make \*Settings::new async for optionally fetching config from a remote url
- relayer checks replica updater addresses match, errors channel if otherwise
- add bootup-only tracing subscriber
- add environment variable overrides for agent configuration
- add tests for agent environment variable overrides
- remove `enabled` flag from agents project-wide

### agents@1.0.0

- bumps version for first release
- adds a changelog
