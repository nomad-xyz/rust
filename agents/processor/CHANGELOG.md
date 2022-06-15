# Changelog

### Unreleased

- refactor: processor now uses global AWS client when proof pushing is enabled
- prevent processor from retrying messages it has previously attempted to
  process
- improve prove/process tracing
- add environment variable overrides for agent configuration
- add tests for agent environment variable overrides
- remove `enabled` flag from agents project-wide

### 1.0.0

- bumps version for first release
- adds a changelog
