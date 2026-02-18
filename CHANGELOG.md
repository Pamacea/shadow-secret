# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.6] - 2026-02-18

### Fixed

- **update command**: Fixed cross-platform npm command execution
  - Now uses `which` crate to find npm executable on all platforms
  - Resolves issue where `npm view` failed on Windows (npm.cmd not found)
  - Works correctly on Windows, Linux, and macOS

### Changed

- **License**: Changed from MIT to GPL-3.0
  - All derivatives must now be shared under the same license
  - Prevents proprietary use of the codebase

## [0.5.5] - 2026-02-18

### Changed

- **Config file naming**: Project configuration renamed from `global.yaml` to `project.yaml`
  - Clarifies distinction between project-specific and global configurations
  - Global config remains at `~/.config/shadow-secret/global.yaml`
  - Project config is now `project.yaml` in project root

### Fixed

- **init-project**: Now correctly creates `project.yaml` configuration file
  - Previously, the file creation was silently failing
  - All configuration files now properly created during initialization

### Changed

- **Default config path**: `unlock` and `push-cloud` now default to `project.yaml`
- **Doctor command**: Updated to check for `project.yaml` instead of `global.yaml`
- **Config module**: Updated `from_current_dir()` to look for `project.yaml`
- **Vercel integration**: Updated to read `vercel_project_id` from `project.yaml`

## [0.5.4] - 2026-02-18

### Fixed

- **Code quality**: Fixed clippy warnings and formatting issues
  - CRLF line endings normalized to LF
  - Minor code style improvements

## [0.5.3] - 2026-02-17

### Fixed

- **Global config path**: `add_to_global_config` now uses correct path `~/.config/shadow-secret/global.yaml`
  - Previously looked for `~/.global.yaml` which doesn't match `init-global` output
  - Error message now correctly points to `shadow-secret init-global` command

### Changed

- **Tests**: Updated integration tests to use correct global config directory structure

## [0.5.2] - 2026-02-17

### Fixed

- **init-project**: Now creates `global.yaml` configuration file
  - Previously, `init-project` only created `.sops.yaml` and `.enc.env`
  - Users had to manually create `global.yaml` before running `unlock`
  - Now includes a template configuration with example targets
  - Configuration includes inline documentation for customization

### Changed

- **init-project workflow**: Added Step 5 to create project configuration
- **Next steps**: Updated instructions to mention editing `global.yaml` first

## [0.5.1] - 2026-02-17

### Fixed

- **JSON/YAML injection**: Fixed placeholder replacement to preserve original formatting and key order
  - Previously, JSON files were re-serialized with keys sorted alphabetically
  - YAML files were also re-serialized, losing original formatting
  - Now uses simple text replacement that preserves exact structure
  - This fixes issues where services failed to read configuration files after injection
  - No more "key not found" errors due to reordering

### Technical Details

- Removed `replace_placeholders_json()` and `replace_placeholders_yaml()` functions
- Now uses `replace_placeholders()` for all file types (JSON, YAML, ENV)
- This approach preserves:
  - Key order in objects
  - Original indentation and formatting
  - Comments in YAML files
  - All structural elements

## [0.5.0] - 2025-02-16

### Added

- **Update command**: `shadow-secret update` for easy package updates
  - Automatically checks for new versions on NPM
  - One-command update: `npm install -g @oalacea/shadow-secret@latest`
  - `--check-only` flag to check updates without installing
  - No more manual uninstall/reinstall cycles

## [0.4.3] - 2025-02-16

### Changed

- **GitHub Actions**: Improved CI and Coverage workflows
  - Consolidated cargo caching with better key structure
  - Added `fail-fast: false` to matrix strategies
  - Improved artifact handling with `if-no-files-found: warn`
  - Better error handling for coverage uploads

### Fixed

- CI workflow now uses consolidated caching
- Coverage workflow now handles upload failures gracefully

## [0.4.2] - 2025-02-16

### Fixed

- **GitHub Actions**: Fixed NPM publish workflow
  - Corrected artifact download paths with `merge-multiple: false`
  - Added Node.js setup step before publishing
  - Added binary verification step
  - Proper chmod for all platform binaries
  - Updated softprops/action-gh-release to v2

### Changed

- **Publish workflow**: More robust artifact handling
  - Uses `pattern: binary-*` for artifact download
  - Better error handling with `if-no-files-found: error`
  - Added `fail-fast: false` to matrix strategy

## [0.4.1] - 2025-02-16

### Fixed

- **JSON/YAML parsing**: Fixed BOM (Byte Order Mark) handling in JSON and YAML files
  - Files with UTF-8 BOM (`\uFEFF`) now parse correctly
  - Error "expected value" at line 1, column 1 resolved
  - BOM stripping applied before parsing in `replace_placeholders_json()` and `replace_placeholders_yaml()`

## [0.4.0] - 2025-02-16

### Changed

- **Workflow**: `unlock` and `unlock-global` now wait for **Enter key** instead of Ctrl+C
- **Behavior**: Press Enter to restore templates and exit (more intuitive than Ctrl+C)
- **User Experience**: Clear "Press Enter to lock secrets" prompt

### Fixed

- Removed Ctrl+C signal handlers (no longer needed)
- Simplified session termination with keyboard input

## [0.3.9] - 2026-02-16

### Fixed

- **Doctor command** now auto-detects global vs project mode
  - No longer fails with "global.yaml not found" when using `unlock-global`
  - Suggests `unlock-global` when only global config exists
  - Added helpful hints for missing `age_key_path` field

- **Improved diagnostics** for missing `age_key_path` configuration
  - Doctor now checks if `age_key_path` exists in config
  - Provides clear instructions when field is missing
  - Suggests proper YAML syntax for adding the field

- **Documentation** added for `age_key_path` migration
  - See `docs/AGE_KEY_PATH_MIGRATION.md` for upgrade guide
  - Helps users update from older global.yaml format

### Changed

- Doctor now runs basic checks (sops, age, SOPS_AGE_KEY_FILE) even in global mode
- Better error messages guide users to fix configuration issues

## [0.3.8] - 2026-02-16

### Added

- `unlock-global` command for explicit global secret unlocking
- `vault_path` field to `VaultConfig` for explicit vault location
- Support for encrypted drives with absolute paths (Windows, macOS, Linux)
- Comprehensive error messages for path resolution issues
- Tilde expansion (`~`) support in vault paths

### Changed

- `vault_source_path()` now resolves relative paths from config directory (not CWD)
- `unlock` command now only loads project-specific config (no global fallback)
- Improved path resolution with `~` expansion support
- Enhanced error messages suggest `unlock-global` when appropriate

### Fixed

- Path resolution bug where global vault was searched in current directory
- Encrypted drive support by allowing explicit `vault_path`
- Confusing behavior where `unlock` tried both project and global configs

### Migration

If you were using `unlock` for global secrets:
  Before: `shadow-secret unlock` (tried both)
  After: `shadow-secret unlock-global` (explicit)

## [0.3.7] - 2026-02-16

### Added

- `age_key_path` field in global.yaml for automatic SOPS key detection
- Shadow Secret now automatically sets `SOPS_AGE_KEY_FILE` environment variable when calling SOPS
- No manual environment variable configuration needed anymore

### Fixed

- SOPS encryption now works reliably with `--output` flag for in-place encryption
- Fixed file extension matching in SOPS regex patterns
- Enhanced documentation in global.yaml with automatic key path configuration

## [0.3.5] - 2026-02-16

### Added

- `init-global` command for centralized secret management
- Automatic fallback to global configuration when project-specific config is missing
- Global configuration directory: `~/.config/shadow-secret/`
- Support for moving configuration to encrypted drives (e.g., VeraCrypt volumes)
- Enhanced config discovery with clear messaging when using global config

### Changed

- `unlock` command now automatically falls back to `~/.config/shadow-secret/global.yaml` if `global.yaml` not found in current directory
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
