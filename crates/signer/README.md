# X-Road Signer (`signer`)

`signer` is the cryptographic key management and signing engine for the Security Server. It isolates private keys, manages software and hardware tokens, and exposes signing primitives via Tonic gRPC.

## Features

- **Token & Key Management**:
  - Software tokens (`soft-token-0`) with file-backed encrypted key storage.
  - Dynamic PKCS#11 module integration for Hardware Security Modules (HSM, e.g. SafeNet Luna, Utimaco, SoftHSM2).
- **Cryptographic Operations**:
  - RSA 2048/3072/4096-bit and ECDSA NIST P-256 / SECP384R1 key generation.
  - Digital signatures over SHA-256 and SHA-512 message digests.
  - PKCS#10 Certificate Signing Request (CSR) generation with custom Subject Distinguished Names.
- **Tonic gRPC Service**: Exposes `SignerService` for `ClientProxy`, `ServerProxy`, and `SecurityServer` admin services.
