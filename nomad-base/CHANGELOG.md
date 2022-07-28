# Changelog

### Unreleased

- implement display for home and replica enums
- add home and remote labels to contract sync metrics for event differentiation
- add `CONFIG_URL` check to `decl_settings` to optionally fetch config from a remote url
- prometheus metrics accepts port by env var
- bug: add checks for empty replica name arrays in `NomadAgent::run_many` and
  `NomadAgent::run_all`
- add `previously_attempted` to the DB schema
- remove `enabled` flag from agents project-wide
- adds a changelog
