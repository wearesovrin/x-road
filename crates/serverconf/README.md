# X-Road Server Configuration (`serverconf`)

`serverconf` manages the local configuration database of a Security Server, storing information about local clients, registered services, access rights, and API keys.

## Architecture & Database

- **Database Engine**: Backed by **SeaORM** (supporting SQLite and PostgreSQL).
- **Module Layout**:
  - `database/entities/`: Strongly-typed SeaORM entity models (`client`, `service`, `endpoint`, `access_right`, `certificate`, `local_group`, `group_member`, `api_key`, `api_key_role`).
  - `database/migrations/`: Automated SeaORM schema migrations (`Migrator::up`).
- **Services**:
  - `ClientService`: Manages local member and subsystem registrations.
  - `ServiceService` & `EndpointService`: Manages REST OpenAPI3 / SOAP WSDL service endpoints, URLs, and timeouts.
  - `AccessRightService`: Enforces access control lists (ACL) for caller subsystems and local groups.
  - `ApiKeyService`: Manages Security Server Admin API keys, roles, and permissions.
