# K7s

K7s is a lightweight, opinionated container orchestration service inspired by Kubernetes, but purposefully simplified.
Instead of managing containers directly, k7s maintains and applies a local `docker-compose.yml` and exposes an HTTP API to apply changes.

### Features
- Minimal HTTP API built with Axum
- Applies `docker compose up -d` on demand via a secure endpoint
- Password-based authorization via HTTP `Authorization` header
- Structured JSON logging with timestamp and source location
- File-based configuration only (no env vars), with safe defaults and bootstrap

### API
- PUT `/apply`
  - Re-runs the equivalent of `docker compose up -d` (fallback to `docker-compose up -d`).
  - Authorization:
    - `Authorization: Bearer <password>`
    - or `Authorization: <password>`
  - Response shape (success):
    ```json
    {"code":0, "msg":"ok", "data": {"status": "success", "response": "<docker stdout>"}}
    ```
  - Response shape (failure, HTTP 500):
    ```json
    {"code":-1, "msg":"apply_failed", "data": {"status": "failed", "response": "<docker stderr>"}}
    ```

### Configuration
k7s reads configuration from a file (one of the following):
- `k7s.yaml`
- `k7s` (YAML without extension)
- `config/k7s.yaml`
- `config/k7s`

If no configuration file is found at startup, k7s will generate a sample `k7s.yaml` in the current directory and exit with a non-zero status.

Example `k7s.yaml`:
```yaml
server:
  host: "127.0.0.1"
  port: 18080
auth:
  password: "your-strong-password"
```

### Logging
k7s prints JSON logs to stdout, with:
- `timestamp` (UTC, RFC3339)
- `level`
- `target`
- `location` (relative path under `src/` + line number if available)
- `message`
- `fields`

Example:
```json
{"timestamp":"2025-08-12T20:59:14Z","level":"INFO","target":"k7s","location":"src/main.rs:25","message":"Starting server on 127.0.0.1:18080","fields":{}}
```

### Quick Start
1. Ensure Docker is installed and running.
2. Create a `docker-compose.yml` in the project root.
   - Example:
     ```yaml
     services:
       hello:
         image: hashicorp/http-echo:0.2.3
         command: ["-text=hello world"]
         ports:
           - "8081:5678"
         restart: unless-stopped
     ```
3. Configure `k7s.yaml` with your desired host/port and authorization password.
4. Run the server:
   ```bash
   cargo run
   ```
5. Apply your compose changes:
   ```bash
   curl -X PUT "http://<host>:<port>/apply" -H "Authorization: Bearer <password>"
   ```

### Security Notes
- Use a strong password in `k7s.yaml`. All callers with the password can trigger `docker compose` on this host.
- Consider running k7s behind a firewall or reverse proxy with additional access controls.

### License
MIT


