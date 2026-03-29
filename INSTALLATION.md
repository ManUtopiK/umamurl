# Installation & Configuration

## Building from source

```bash
cargo build --release --manifest-path=actix/Cargo.toml
```

The binary will be at `actix/target/release/umamurl`.

## Running

```bash
export password=changeme
export db_url=urls.sqlite
./umamurl
```

Or during development:

```bash
cargo run --manifest-path=actix/Cargo.toml
```

## Configuration

All configuration is done via environment variables. Variables marked with `#` are important.

### `db_url` #

Database file location. Defaults to `urls.sqlite`. See [`use_wal_mode`](#use_wal_mode-) before changing.

### `password` #

Admin password. Can be set in two ways:

1. **Environment variable** -- set `password` before starting. Takes priority.
2. **UI setup** -- if no password is configured, the web UI will prompt you to set one on first visit. The password is hashed with Argon2 and stored in the database.

If no password is set at all, the instance is open to anyone.

Password is not encrypted in transport. Use a reverse proxy like [Caddy](https://caddyserver.com/) for SSL.

### `site_url` #

Your public-facing URL (e.g. `https://short.example.com`). Optional. Used for clipboard copy and QR code generation. Do not quote or add a trailing slash. Unicode domains should use punycode.

### `umami_url` and `umami_website_id`

Enable [Umami](https://umami.is/) analytics on redirects. Can be set in two ways:

1. **Environment variables** -- set both `umami_url` and `umami_website_id`. Takes priority and cannot be changed from the UI.
2. **Admin UI** -- click the gear icon in the header to configure. Values are stored in the database.

Both values must be set for analytics to work. Events are sent server-side on each redirect (fire-and-forget).

Example:

```bash
export umami_url=https://analytics.example.com
export umami_website_id=a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

### `api_key`

API key for programmatic access. A weak key triggers a warning with a generated alternative.

Generate a secure key: `tr -dc A-Za-z0-9 </dev/urandom | head -c 128`

### `use_wal_mode` #

Set to `True` to enable [WAL journal mode](https://sqlite.org/wal.html). Highly recommended. Gives a significant performance boost under load since writes no longer block reads. Also enables automatic database backups.

Make sure `db_url` points to a file inside a directory (not a bare file mount).

### `ensure_acid`

Database is [ACID-compliant](https://www.slingacademy.com/article/acid-properties-in-sqlite-why-they-matter) by default. Set to `False` to trade durability for throughput. Data loss is only possible on system failure or power loss.

### `redirect_method` #

`PERMANENT` (308, default) or `TEMPORARY` (307).

### `slug_style`

`Pair` (default, e.g. `gifted-ramanujan`) or `UID` (random characters).

### `slug_length`

UID slug length. Minimum 4, default 16. Use 16+ for large link collections.

### `try_longer_slug`

Set to `True` to retry with a longer UID (+4 chars) on collision.

### `listen_address`

Bind address. Defaults to `0.0.0.0`.

### `port`

Listen port. Defaults to `4567`.

### `allow_capital_letters`

Set to `True` to allow A-Z in shortlinks and UID slugs.

### `hash_algorithm` #

Set to `Argon2` to accept hashed password and API key values.

Hash a password:

```bash
echo -n <password> | argon2 <salt> -id -t 3 -m 16 -l 32 -e
```

### `public_mode`

Set to `Enable` to let anyone create links. Listing and deleting still require the password.

### `public_mode_expiry_delay`

When public mode is on, force a maximum expiry (in seconds). Users can choose shorter. No effect for admins.

### `disable_frontend`

Set to `True` to disable the web UI entirely (API-only mode).

### `custom_landing_directory`

Path to a directory with an `index.html` to serve as landing page. The admin dashboard moves to `/admin/manage`.

### `cache_control_header`

Custom `Cache-Control` header value. Example: `no-cache, private`. Not set by default.
