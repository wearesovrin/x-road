# X-Road Configuration Client (`confclient`)

`confclient` implements the background synchronization daemon that fetches, verifies, and caches global configuration from a Central Server.

## Features

- **Configuration Anchor**: Parses and verifies signed configuration anchor files (`configuration-anchor.xml`).
- **Cryptographic Verification**:
  - Multi-location HTTP failover download.
  - Validates downloaded configuration parts against SHA-224/SHA-256 certificate hashes embedded in the anchor.
- **Local Cache Management**:
  - Writes valid configuration files to a local directory (`/etc/xroad/globalconf`).
  - Implements atomic file swapping to guarantee zero-downtime cache updates for local Security Server proxies.
