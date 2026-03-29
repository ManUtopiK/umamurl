# CLI & API Usage

The API supports two authentication methods: **API key** (recommended for scripts) and **cookie** (session-based).

In all examples, replace `http://localhost:4567` with your instance URL.

## API Key Authentication

Set the `api_key` env var on the server. All API key responses are JSON.

Generate a secure key:

```bash
tr -dc A-Za-z0-9 </dev/urandom | head -c 128
```

## Endpoints

### `POST /api/new` -- Create a short link

```bash
curl -X POST \
  -H "X-API-Key: <KEY>" \
  -H "Content-Type: application/json" \
  -d '{"shortlink":"my-link", "longlink":"https://example.com", "expiry_delay": 3600}' \
  http://localhost:4567/api/new
```

- `shortlink`: optional, auto-generated if empty
- `expiry_delay`: seconds, 0 or omitted = no expiry

### `GET /api/all` -- List all links

```bash
curl -H "X-API-Key: <KEY>" http://localhost:4567/api/all
```

Query params: `page_size` (default 10), `page_after` (shortlink offset), `page_no` (page number).

### `PUT /api/edit` -- Edit a link

```bash
curl -X PUT \
  -H "X-API-Key: <KEY>" \
  -H "Content-Type: application/json" \
  -d '{"shortlink":"my-link", "longlink":"https://new-url.com"}' \
  http://localhost:4567/api/edit
```

### `DELETE /api/del/{shortlink}` -- Delete a link

```bash
curl -X DELETE -H "X-API-Key: <KEY>" http://localhost:4567/api/del/my-link
```

### `POST /api/expand` -- Get link info

```bash
curl -X POST -H "X-API-Key: <KEY>" -d 'my-link' http://localhost:4567/api/expand
```

Returns `longurl` and `expiry_time`. API key only (no cookie auth).

### `GET /api/getconfig` -- Backend config

```bash
curl -H "X-API-Key: <KEY>" http://localhost:4567/api/getconfig
```

Returns version, site_url, slug settings, public mode config, umami status.

### `GET /api/whoami` -- Current role

```bash
curl -H "X-API-Key: <KEY>" http://localhost:4567/api/whoami
```

Returns `admin`, `public`, or `nobody`.

### `GET /api/umami-config` -- Umami settings (admin)

```bash
curl -H "X-API-Key: <KEY>" http://localhost:4567/api/umami-config
```

Returns `umami_url`, `umami_website_id`, and `env_configured` (true if set via env vars).

### `POST /api/umami-config` -- Update Umami settings (admin)

```bash
curl -X POST \
  -H "X-API-Key: <KEY>" \
  -H "Content-Type: application/json" \
  -d '{"umami_url":"https://analytics.example.com", "umami_website_id":"your-id"}' \
  http://localhost:4567/api/umami-config
```

Returns 403 if env vars are set (cannot override from API).

### `POST /api/set-password` -- Set password (first-time setup)

```bash
curl -X POST -d 'my-secure-password' http://localhost:4567/api/set-password
```

Only works when no password is configured (env var or DB). Password is hashed with Argon2.

## Cookie Authentication

For session-based access, log in first:

```bash
curl -X POST -d "<password>" -c cookie.txt http://localhost:4567/api/login
```

Then add `-b cookie.txt` to subsequent requests. Sessions expire after 14 days.

Log out:

```bash
curl -X DELETE -b cookie.txt http://localhost:4567/api/logout
```
