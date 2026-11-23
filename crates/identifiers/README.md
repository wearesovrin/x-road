# X-Road Identifiers (`identifiers`)

`identifiers` implements strict, canonical X-Road identifier structures adhering to the W3C XML and JSON specifications for X-Road 6.x and 7.x.

## Core Identifiers

- **`ClientId`**: Identifies members and subsystems (`Instance / MemberClass / MemberCode [/ SubsystemCode]`).
- **`ServiceId`**: Identifies specific services offered by subsystems (`Instance / MemberClass / MemberCode / SubsystemCode / ServiceCode [/ ServiceVersion]`).
- **`SecurityServerId`**: Identifies physical Security Server nodes (`Instance / MemberClass / MemberCode / ServerCode`).
- **`GlobalGroupId`**: Identifies federated global access groups.
- **`CentralServiceId`**: Identifies central services within an X-Road instance.

## Features

- Strict character validation (`StrictIdentifierValidator`) rejecting invalid characters (slashes, colons, percentages, control characters).
- Full roundtrip serialization for both SOAP XML (`id:XRoadClientIdentifierType`, `id:XRoadServiceIdentifierType`) and JSON REST formats.
- Ergonomic `FromStr` parsing and `Display` formatting (e.g., `DEV/COM/12345678/SUBSYSTEM`).
