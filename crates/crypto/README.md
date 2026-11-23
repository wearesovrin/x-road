# X-Road Crypto (`crypto`)

`crypto` provides standardized cryptographic hashing, signature digest calculations, and cryptographic helpers utilized across the X-Road ecosystem.

## Features

- **Standard Digest Functions**: SHA-256 (`sha256_digest`) and SHA-512 (`sha512_digest`) implementations optimized for non-repudiation audit trails and Merkle hash tree calculations.
- **Hex Utilities**: Fast hex formatting and decoding helpers for message digests and certificate thumbprints.
- **Integration**: Powers `messagelog` batch hashing, `signer` digest preparation, and `asic` file integrity validation.
