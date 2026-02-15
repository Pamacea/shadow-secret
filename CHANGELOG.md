# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-02-15

### Fixed

- Fixed test race condition in `test_multiple_backups` - global backup state now properly reset between tests
- All 50 unit tests now passing (cleaner module: 5/5 tests)
- Improved test isolation with `reset_backups()` helper function
- Fixed Windows binary compilation and inclusion in NPM package

## [0.2.0] - 2026-02-15

### Added

- Monorepo structure separating Rust core logic from NPM distribution
- Cross-platform binary support via native Rust compilation
- Automated CI/CD pipeline for binary injection before NPM publish
- Professional project structure with `packages/core` and `packages/cli-npm`
- Comprehensive documentation and developer guides

### Changed

- **BREAKING**: Restructured to monorepo architecture
- Source code moved from `src/` to `packages/core/src/`
- Rust project now located in `packages/core/`
- NPM wrapper added in `packages/cli-npm/`
- Build artifacts now properly gitignored in both core and CLI packages

### Fixed

- Binaries no longer committed to repository
- Cross-platform binary detection automated
- Terminal inheritance for seamless user experience

## [0.1.0] - 2026-02-15

### Added

- Initial release of Shadow Secret
- `doctor` command for system prerequisites verification
- `unlock` command to inject secrets into target files
- `init-project` command to bootstrap new projects with secret infrastructure
- `push-cloud` command to sync secrets to Vercel environment variables
- SOPS integration with age encryption
- Automatic file restoration on process termination
- In-memory secret handling (no disk persistence)
- Vercel CLI integration for cloud deployment
