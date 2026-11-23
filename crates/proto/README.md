# X-Road Protocol Buffers (`proto`)

`proto` contains the compiled Protocol Buffers definitions and generated Tonic gRPC service traits used for inter-process communication between X-Road daemons.

## Proto Definitions

- `signer_service.proto`: gRPC API for key generation, CSR issuance, and digital signatures.
- `monitor_service.proto`: gRPC API for environmental host metrics (`StatsReq` / `StatsResp`, `SystemMetricsReq` / `SystemMetricsResp`).
- `opmonitor_service.proto`: gRPC API for operational data storage and querying (`StoreOperationalData`, `GetOperationalDataIntervals`, `QueryOperationalData`).
- `common_messages.proto`: Shared Protobuf identifier types (`ClientIdProto`, `ServiceIdProto`).

## Features

- Built using `tonic-prost-build` with automated type generation.
