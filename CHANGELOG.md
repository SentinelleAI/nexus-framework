# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.1] - 2026-02-16

### Added
- **DI Container**: `has<T>()`, `service_count()`, and `Debug` implementation
- **Error Handling**: `service_unavailable()` (503) and `gateway_timeout()` (504) constructors
- **Error Handling**: `with_details()` method for structured error details
- **Error Handling**: `details` field on `ErrorResponse` (skipped when null)
- **Configuration**: `NexusConfig::get<T>(key)` for custom user-defined config keys
- **Extractors**: `ValidatedJson<T>` with user-friendly deserialization error messages
- **Documentation**: Complete README rewrite with feature table, quick start, architecture
- **Documentation**: Getting started guide, 6 topic guides, macro reference
- **Project**: `CHANGELOG.md`, `CONTRIBUTING.md`
- **CI/CD**: GitHub Actions workflow (check, test, clippy, fmt, doc)
- **Tests**: Unit tests for DI container, error handling, configuration, guards

### Changed
- **Refactoring**: Split `nexus_framework/src/lib.rs` (655 lines) into 11 focused modules
- **Refactoring**: Split `nexus_framework_macros/src/lib.rs` (778 lines) into 6 focused modules
- **.gitignore**: Comprehensive rules for Rust, IDE, OS, secrets, coverage
- **Prelude**: Added `ValidatedJson`, `serde_json` re-exports

## [0.1.0] - Initial Release

### Added
- Core framework with Axum integration
- `#[service]` and `#[service_impl]` macros for DI
- `#[controller]` and `#[route]` macros for HTTP endpoints
- `#[model]` macro for data models
- `#[nexus_app]` macro for application bootstrap
- `#[scheduled]` macro for cron jobs
- `DependencyContainer` with `get<T>()` and `try_get<T>()`
- `AppError` with common HTTP status constructors
- `NexusConfig` layered configuration system
- Route guards with `guard_middleware`
- Request logging middleware
- System diagnostics logging
- Built-in `/health` and `/ping` endpoints
