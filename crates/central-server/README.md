# X-Road Central Server (`central-server`)

`central-server` implements the central management daemon responsible for administering the X-Road instance, maintaining the member registry, and distributing trusted global configuration.

## Features

- **OpenAPI Admin API**: Interactive REST API built on **Poem-OpenAPI** with Swagger UI hosted at `/swagger-ui`.
  - Member and subsystem registration, approval, and decommissioning.
  - Security Server registration and authentication certificate management.
  - Global group definition and membership assignment.
  - Approved CA and Approved TSA authority configuration.
  - System parameters management.
- **Global Configuration Generator**: Automatically generates and signs `shared-params.xml` and `private-params.xml` distributed over HTTP/HTTPS.
- **Database Engine**: SeaORM-backed persistence under `src/database/entities/` and `src/database/migrations/`.
- **CLI & Daemon**: Executable via `central-server` binary or `x-road central-server` subcommand.
