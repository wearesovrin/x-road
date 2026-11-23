# X-Road Common (`common`)

`common` provides foundational types, error handling, protocol enums, and utility primitives shared across all X-Road workspace crates.

## Features

- **Error Handling**: Unified `XRoadError` enum using `thiserror` covering networking, database, cryptographic, protocol, configuration, and ASiC container errors, along with the `XRoadResult<T>` type alias.
- **Protocol Enums**: Strongly-typed enums powered by `strum` and `serde`:
  - `MessageProtocol`: SOAP vs REST protocol identification.
  - `ServiceCode` & `MetaServiceCode`: Extensible representation of standard X-Road meta-services (`listMethods`, `allowedMethods`, `getWsdl`, `getOpenapi`, etc.) and custom business services.
  - `ClientStatus` & `ManagementStatus`: Standard X-Road member and subsystem lifecycle states.
  - `TokenType`: Hardware (PKCS#11) vs Software token types.
  - `ServiceType`: SOAP (`WSDL`) vs REST (`OPENAPI3`, `REST`).
