# X-Road Proxy (`proxy`)

`proxy` is the high-performance message mediation gateway. It implements both the **ClientProxy** (consumer outbound gateway) and the **ServerProxy** (provider inbound gateway).

## Architecture

- **Client Proxy (`ClientProxy`)**:
  - Listens on TCP port 8000 for incoming SOAP and REST requests from consumer information systems.
  - Resolves target service provider address and access rights via `ServerConf`.
  - Interacts with `Signer` to digitally sign outgoing message envelopes.
  - Sends requests over mutual TLS to the target Security Server's ServerProxy.
  - Logs transactions to `MessageLog`.
- **Server Proxy (`ServerProxy`)**:
  - Listens on TCP port 8443 (HTTP/HTTPS) and TCP port 5500 (native mTLS socket).
  - Authenticates caller Security Server authentication certificates against `GlobalConf` approved CAs.
  - Validates caller access rights and endpoint authorizations.
  - Executes built-in metaservices (`listMethods`, `allowedMethods`) with zero backend hop.
  - Forwards authorized business requests to provider adapter services and returns the response.
- **Native mTLS Engine**:
  - Binds on TCP port 5500 with `tokio-rustls` and WebPki client certificate verification.
