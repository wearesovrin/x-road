# X-Road Unified Workspace & CLI (`x-road`)

`x-road` is the primary workspace driver and executable umbrella crate for the pure-Rust X-Road ecosystem.

## Binaries

### 1. `x-road` (Daemon Launcher)

Launches any X-Road daemon or unified node:
```bash
# Start unified Security Server node (Admin API, Client Proxy, Server Proxy, mTLS)
x-road security-server --admin-bind 0.0.0.0:4000 --database-url sqlite://serverconf.db

# Start Central Server daemon
x-road central-server --bind 0.0.0.0:4001 --instance DEV

# Start standalone Client Proxy
x-road proxy --bind 127.0.0.1:8000 --client DEV/COM/12345678

# Start Signer key management daemon
x-road signer --bind 127.0.0.1:50051

# Start Configuration Client sync daemon
x-road confclient --instance DEV --url http://central-server/internalconf

# Start Environmental & Operational Monitoring daemon
x-road monitor --bind 127.0.0.1:50052
```

### 2. `xroad-cli` (Operations & Admin CLI)

A comprehensive CLI utility for managing X-Road nodes:
- `xroad-cli status`: Check services status.
- `xroad-cli key [list|generate|csr]`: Software token key management.
- `xroad-cli conf`: Synchronize and inspect global configuration.
- `xroad-cli asic [pack|inspect]`: Inspect and create ASiC-e signature containers.
- `xroad-cli did`: Decentralized identity and DID resolution operations.
- `xroad-cli security-server`: Security Server admin operations.
- `xroad-cli central-server`: Central Server admin operations.
- `xroad-cli messagelog [archive|verify]`: MessageLog archival and verification.
