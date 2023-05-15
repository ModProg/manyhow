# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- ## [Unreleased] -->
## [0.4.2] - 2023-05-15
### Fixed
- `ToTokens`' SpanRange conversion should work without `proc_macro`.

## [0.4.1] - 2023-05-14
### Fixed
- `manyhow_macros` version

## [0.4.0] - 2023-05-14
### Added
- `impl_fn` flag to create separate implementation function types.

## [0.3.0] - 2023-05-02
### Added
- `SpanRanged` implementation for `Option<impl SpanRanged>`.

## [0.2.0] - 2023-04-19
### Changed
- Moved `Error::join` to `JoinToTokensError` trait.

## [0.1.1] - 2023-04-16
Only documentation changes.

## [v0.1.0] 
**Initial Release**

[unreleased]: https://github.com/ModProg/manyhow/compare/v0.4.2...HEAD
[0.4.2]: https://github.com/ModProg/manyhow/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/ModProg/manyhow/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/ModProg/manyhow/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ModProg/manyhow/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ModProg/manyhow/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/ModProg/manyhow/compare/v0.1.0...v0.1.1
[v0.1.0]: https://github.com/ModProg/manyhow/tree/v0.1.0
