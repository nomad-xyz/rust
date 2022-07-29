# Changelog

### Unreleased

- update handler now errors if incoming updates have an unexpected updater
- double-update routine now checks that both updates are signed by the same
  updater
- Add English description to XCM error log, change to use `Display`

### agents@1.1.0

- make \*Settings::new async for optionally fetching config from a remote url
- add bootup-only tracing subscriber
- add environment variable overrides for agent configuration
- add tests for agent environment variable overrides
- remove `enabled` flag from agents project-wide

### agents@1.0.0

- bumps version for first release
- adds a changelog
