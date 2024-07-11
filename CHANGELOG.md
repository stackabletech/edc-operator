# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- Configuration overrides for the JVM security properties, such as DNS caching ([#24]).
- Helm: support labels in values.yaml ([#49]).

### Changed

- Bump `stackable-operator` to `0.70.0`, `product-config` to `0.7.0`, and other dependencies  ([#112]).
- Upgraded to EDC 0.1.2 ([#23]).
- Operator-rs: `0.46.0` ([#20], [#21]).
- Switched to workspace dependencies ([#21]).
- Increase the size limit of the log volumes ([#20]).
- Use new label builders ([#45]).

[#20]: https://github.com/stackabletech/edc-operator/pull/20
[#21]: https://github.com/stackabletech/edc-operator/pull/21
[#23]: https://github.com/stackabletech/edc-operator/pull/23
[#24]: https://github.com/stackabletech/edc-operator/pull/24
[#45]: https://github.com/stackabletech/edc-operator/pull/45
[#49]: https://github.com/stackabletech/edc-operator/pull/49
[#112]: https://github.com/stackabletech/edc-operator/pull/112