# X-Road Message Log (`messagelog`)

`messagelog` provides the secure, non-repudiation logging and verification subsystem for all messages processed by a Security Server.

## Architecture & Database

- **Database Engine**: Backed by **SeaORM** (supporting SQLite and PostgreSQL).
- **Module Layout**:
  - `database/entities/`: SeaORM entity models for `log_record` and `attachment`.
  - `database/migrations/`: Automated schema migrations (`Migrator::up`).
- **Core Components**:
  - **Batch Hasher (`BatchHasher`)**: Implements the Freudenthal Merkle hash chain tree. Links individual message digest hashes into consecutive cryptographic batch roots.
  - **Time-Stamping Service (`TimestampingService`)**: Issues RFC 3161 DER TimeStampReq HTTP queries to external Time-Stamping Authorities (TSAs) to acquire trusted time-stamps.
  - **Archiver (`MessageLogArchiver`)**: Background task that groups verified message batches into standardized ZIP archives containing signed `.asice` containers.
  - **Verifier (`verify_archive`)**: Verifies archive integrity, Freudenthal hash step continuity, and signatures.
