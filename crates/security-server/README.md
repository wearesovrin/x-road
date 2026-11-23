# X-Road Security Server (`security-server`)

`security-server` is the complete unified Security Server node daemon. It brings together the Admin API, Client Proxy, Server Proxy, mTLS listener, and background workers into a single executable service.

## Architecture

- **Security Server Node (`SecurityServerNode`)**:
  - Unifies `ServerConf` database, `MessageLog` audit trail, `GlobalConf`, and `Signer`.
  - Concurrently spawns:
    - **Admin API**: Port 4000 (Poem-OpenAPI with Swagger UI at `/swagger-ui`).
    - **Client Proxy**: Port 8000 (outgoing consumer gateway).
    - **Server Proxy**: Port 8443 (incoming provider gateway).
    - **Native mTLS Listener**: Port 5500 (wire mutual TLS handshake).
- **Static Admin UI Hosting**:
  - Built-in support for serving the Vue.js Single-Page Application (SPA) web frontend from static directories (`--ui-static-path` / `XROAD_UI_PATH`).
- **Configuration & CLI**:
  - Can be run directly via `security-server` or as `x-road security-server`.
  - Full configuration via command line flags or environment variables (`ADMIN_BIND_ADDR`, `DATABASE_URL`, `MESSAGELOG_DATABASE_URL`).
