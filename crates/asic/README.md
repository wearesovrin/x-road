# X-Road ASiC-e (`asic`)

`asic` implements the Associated Signature Containers Extended (ASiC-e) profile and XAdES-BES digital signatures used for non-repudiation logging in X-Road.

## Features

- **W3C XML Canonicalization (C14N)**:
  - Canonical XML 1.0 (Inclusive) and Exclusive XML Canonicalization (REC-xml-exc-c14n-20020718) implemented via the pure-Rust [`xml-sec`](https://crates.io/crates/xml-sec) crate.
  - Normalizes XML namespaces, attributes, and line breaks to ensure bit-for-bit signature validation against upstream Java Security Servers.
- **ASiC-e Container Packing & Extraction**:
  - Creates compliant ZIP-based `.asice` containers containing `mimetype` (`application/vnd.etsi.asic-e+zip`), payload documents, `META-INF/manifest.xml`, and `META-INF/signatures.xml`.
- **XAdES-BES Signatures**:
  - Generates and verifies XML-DSig / XAdES-BES signatures containing `SignedProperties`, `SigningTime`, `SigningCertificate`, and digest references.
- **CLI Tool**:
  - Ships the `asic-tool` binary for inspecting and packaging `.asice` files from the command line.
