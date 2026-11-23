# X-Road ACME (`acme`)

`acme` provides automated certificate management via the Automated Certificate Management Environment (ACME / RFC 8555) protocol.

## Features

- **Automated Certificate Issuance**: Integrates with Let's Encrypt and private ACME CAs (e.g., Smallstep, HashiCorp Vault).
- **Challenge Handlers**: Supports HTTP-01 challenge response store with automatic token serving on port 80.
- **Background Renewal Worker**: Continuous monitoring worker that tracks certificate validity periods and triggers automatic re-issuance before expiration.
- **Security Server Integration**: Issues and renews Security Server TLS and authentication certificates seamlessly.
