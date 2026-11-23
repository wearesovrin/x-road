# X-Road Global Configuration (`globalconf`)

`globalconf` manages the parsed in-memory global configuration directory distributed across all members within an X-Road instance and federated partner instances.

## Features

- **Configuration Parser**: Reads and parses `shared-params.xml` and `private-params.xml`.
- **Approved Trust Services**:
  - Approved Certification Authorities (CAs) and intermediate issuers.
  - Approved Time-Stamping Authorities (TSAs) for RFC 3161 timestamps.
  - OCSP responder URLs and TLS certificate verification settings.
- **Member Directory**:
  - Fast lookup of member names, classes, codes, and registered subsystems.
  - Security Server registry: maps `SecurityServerId` to IP addresses and public authentication certificates.
- **Security Policy Enforcement**:
  - Global group membership resolution.
  - Security Server mutual authentication certificate validation against CA roots.
