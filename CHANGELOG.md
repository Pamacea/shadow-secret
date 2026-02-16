# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.5] - 2026-02-16

### Added

- `init-global` command for centralized secret management
- Automatic fallback to global configuration when project-specific config is missing
- Global configuration directory: `~/.config/shadow-secret/`
- Support for moving configuration to encrypted drives (e.g., VeraCrypt volumes)
- Enhanced config discovery with clear messaging when using global config

### Changed

- `unlock` command now automatically falls back to `~/.config/shadow-secret/global.yaml` if `shadow-secret.yaml` not found in current directory
- Improved documentation with global configuration examples and usage patterns

### Fixed

- GitHub Actions workflows updated for monorepo structure (CI, Coverage, Publish)
- All workflows now use correct paths (`packages/core/`)

## [0.3.3] - 2026-02-15

### Fixed

- Fixed GitHub Actions artifact download and organization
- Fixed binary naming for macOS (x64 and ARM64 now use distinct names)
- Updated bridge.js with proper platform detection for macOS architectures
- Removed obsolete "update bridge.js" step from workflow
- All binaries now correctly placed in bin/ directory with proper permissions

## [0.3.2] - 2026-02-15

### Fixed

- Fixed GitHub Actions workflow for multi-platform builds
- Added Windows x64 to build matrix (was missing)
- Removed problematic "rename binary" step that caused "same file" error
- Separated workflow into 3 jobs: build, publish, create-release
- Fixed artifact download and executable permissions handling
- All 4 platforms now build in parallel (Windows, Linux x64, macOS x64/ARM64)

## [0.3.1] - 2026-02-15

### Fixed

- Fixed compilation errors on Unix/macOS platforms
- Changed `std::os::unix::fs::Permissions` to `std::fs::Permissions` (public API)
- Added `.clone()` when restoring file permissions to fix ownership issue
- All platforms (Windows, Linux, macOS) now compile successfully

## [0.3.0] - 2026-02-15

### Added

- Multi-platform support (Windows, Linux x64, macOS x64/ARM64)
- Automated CI/CD pipeline via GitHub Actions
- Automatic binary compilation for all platforms on release
- Single NPM package includes all platform binaries
- Improved NPM package structure (binaries now properly included)

### Changed

- **BREAKING**: Package renamed from `shadow-secret` to `@oalacea/shadow-secret`
- Binaries now compiled automatically via GitHub Actions
- Platform detection fully implemented in bridge.js

### Fixed

- Fixed NPM package to correctly include compiled binaries
- Removed `bin/.gitignore` (prevented binaries from being packaged)
- Added proper `.gitignore` at package level instead

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
