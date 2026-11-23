# X-Road Message (`message`)

`message` implements SOAP and REST wire protocol parsing, X-Road header extraction, envelope generation, and MIME multi-part handling.

## Features

- **X-Road Header Parsing**:
  - Extracts canonical headers: `client`, `service`, `userId`, `id` (query ID), `protocolVersion`, `issue`, and `representedParty`.
  - Supports both XML SOAP 1.1/1.2 headers and HTTP REST headers (`X-Road-Client`, `X-Road-Service`, etc.).
- **SOAP Envelope Serialization**:
  - `SoapEnvelopeXml` builder and parser with full XML namespace support (`http://x-road.eu/xsd/xroad.xsd`, `http://schemas.xmlsoap.org/soap/envelope/`).
  - Standardized SOAP Fault generation (`Server.ServerProxy.UnknownService`, `Client.InvalidHeader`, etc.).
- **Meta-Service Responses**:
  - Formats built-in meta-service responses (`listMethods`, `allowedMethods`) into compliant XML or JSON.
