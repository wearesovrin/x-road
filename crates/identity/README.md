# X-Road Identity (`identity`)

`identity` provides decentralized identity resolution, W3C DID document parsing, and Verifiable Credential verification for modern interoperability extensions in X-Road.

## Features

- **W3C DID Documents**: Supports parsing and resolution of `did:xroad:...`, `did:key:...`, and `did:web:...` identifiers.
- **Verification Methods**: Extracts public keys and verification relationships (authentication, assertionMethod).
- **Verifiable Credentials (VC)**: Validates JSON-LD and JWT formatted verifiable credentials presented by clients.
- **Identity Resolver**: Pluggable resolver trait with in-memory caching and resolution fallbacks.
