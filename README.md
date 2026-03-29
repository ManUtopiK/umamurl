<!-- SPDX-FileCopyrightText: 2023 Emmanuel Salomon <emmanuel.salomon@gmail.com> -->
<!-- SPDX-License-Identifier: MIT -->

[![github-tests-badge](https://github.com/ManUtopiK/umamurl/actions/workflows/rust-tests.yml/badge.svg)](https://github.com/ManUtopiK/umamurl/actions/workflows/rust-tests.yml)
[![license-badge](https://img.shields.io/github/license/ManUtopiK/umamurl)](https://spdx.org/licenses/MIT.html)
[![latest-release-badge](https://img.shields.io/github/v/release/ManUtopiK/umamurl?label=latest%20release)](https://github.com/ManUtopiK/umamurl/releases/latest)

# ![Logo](resources/assets/favicon-32.png) Umamurl

A fast, lightweight URL shortener with built-in [Umami](https://umami.is/) analytics. Self-hosted, privacy-friendly, no bloat.

## Why Umamurl?

Most URL shorteners ship with features you don't need. Umamurl takes a different approach:

- **Umami analytics integration** -- monitor redirects with your own self-hosted Umami instance
- **Tiny footprint** -- single binary, uses <15 MB RAM
- **Fast** -- Rust backend, SQLite database, instant redirects (no interstitial pages)
- **Simple to deploy** -- single binary, env vars for config

## Features

| Feature                       | Details                                                                     |
| ----------------------------- | --------------------------------------------------------------------------- |
| URL shortening                | Random (adjective-name pairs or UIDs) or custom slugs                       |
| Umami analytics               | Optional, configurable via env vars or admin UI                             |
| Link expiry                   | Automatic expiry after a chosen duration                                    |
| Edit & delete                 | Modify or remove links after creation                                       |
| QR codes                      | One-click generation for any short link                                     |
| Password protection           | Set via env var or configure from the UI on first launch                    |
| API key support               | For CLI and programmatic access, supports Argon2 hashed credentials         |
| Public mode                   | Let anyone shorten links, admin-only listing and deletion                   |
| Custom landing page           | Serve your own HTML, admin dashboard moves to `/admin/manage`               |
| Mobile-friendly dark UI       | Responsive design with Tailwind CSS                                         |
| ACID-compliant SQLite         | WAL mode recommended for performance, configurable durability               |

## Quick start

```bash
# Set minimal config
export password=changeme
export db_url=urls.sqlite

# Run
cargo run --manifest-path=actix/Cargo.toml
```

Open `http://localhost:4567`. On first launch without a password env var, the UI will prompt you to set one.

Umami analytics can be configured from the admin settings (gear icon) or via env vars:

```bash
export umami_url=https://your-umami.example.com
export umami_website_id=your-website-id
```

## Documentation

- [Installation & configuration](./INSTALLATION.md) -- all env vars, building from source
- [CLI & API usage](./CLI.md) -- curl examples, API key setup

## What will NOT be added

- **Umami integration is opt-in and self-hosted.** No third-party services involved.
- **User management.** One password, one instance.
- **Cookies, popups, newsletters.** None of that.

## Notes

- Started as a fork of [`simply-shorten`](https://gitlab.com/draganczukp/simply-shorten).
- Adjective-name pairs come from [Moby's name generator](https://github.com/moby/moby/blob/master/pkg/namesgenerator/names-generator.go).
- [Enable WAL mode](./INSTALLATION.md#use_wal_mode-) for best performance.
- For >1000 links, use UID `slug_style` with `slug_length` of 16+.
