# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-10-07

### Changed

- Linux builds produce a static binary linked against musl. Its runtime closure
  is the binary and its resources only: no glibc, no OpenSSL, nothing pulled in
  from the flake's nixpkgs pin.
- Outbound HTTPS (Umami events) uses rustls instead of OpenSSL. CA roots are
  compiled in (webpki-roots), so the host trust store is no longer read. An
  Umami instance behind a private CA is not trusted anymore.
- mimalloc is the global allocator, since musl's allocator is slow under
  concurrent load.
- Umami events reuse a single HTTP client, instead of building one (TLS config
  and connection pool) on every redirect.
- The flake reads its version from `actix/Cargo.toml`.
- Dependencies updated: actix-web 4.15, actix-files 0.7, rusqlite 0.40,
  argon2 0.6, nanoid 0.5, tokio 1.53 and all compatible transitive crates.
  Password hashes stored by previous versions still validate.
- `rust-version` is set to 1.97, the toolchain of the Nix build.

### Removed

- Server-side HTTP/2. The server binds plain TCP without TLS, so it only ever
  served HTTP/1.1 and this code was unreachable.
- `x86_64-darwin` from the flake's systems: the pinned nixpkgs no longer
  supports it and failed to evaluate.

### Security

- rustls: RUSTSEC-2026-0285.
- rustls-webpki: RUSTSEC-2026-0049, RUSTSEC-2026-0098, RUSTSEC-2026-0099,
  RUSTSEC-2026-0104.
- h2: RUSTSEC-2026-0258 (0.4 updated, 0.3 removed along with HTTP/2).
- Unsoundness warnings in rand (RUSTSEC-2026-0097) and anyhow
  (RUSTSEC-2026-0190), and the yanked chacha20 release.

## [1.0.0] - 2026-08-18

First version, not tagged at the time.

### Added

- URL shortener served by actix-web, with a SQLite store and an admin UI.
- Built-in Umami analytics: each redirect sends an event to an Umami instance.
- Nix flake, with prebuilt binaries published to `cache.serveur-ia.fr`.

[Unreleased]: https://github.com/ManUtopiK/umamurl/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/ManUtopiK/umamurl/compare/51a5ea4...v1.1.0
[1.0.0]: https://github.com/ManUtopiK/umamurl/tree/51a5ea4
