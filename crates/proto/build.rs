fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = [
        "proto/common_messages.proto",
        "proto/error_handling.proto",
        "proto/tokens.proto",
        "proto/token_status_info.proto",
        "proto/key_service.proto",
        "proto/token_service.proto",
        "proto/certificate_service.proto",
        "proto/ocsp_service.proto",
        "proto/globalconf_service.proto",
        "proto/anchor_service.proto",
        "proto/monitor_service.proto",
        "proto/admin_service.proto",
        "proto/opmonitor_service.proto",
    ];

    let mut includes = vec!["proto"];
    if std::path::Path::new("/usr/include").exists() {
        includes.push("/usr/include");
    }
    if std::path::Path::new("/usr/local/include").exists() {
        includes.push("/usr/local/include");
    }

    tonic_prost_build::configure().compile_protos(&protos, &includes)?;

    Ok(())
}
