fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile XCBBuildService proto files
    tonic_build::configure()
        .build_server(true)
        .build_client(false) // We only need the server side
        .compile_protos(
            &["proto/xcode/cas.proto", "proto/xcode/keyvalue.proto"],
            &["proto/xcode"],
        )?;

    // Compile Bazel Remote Execution API proto files
    // Use our custom google.rpc.Status to avoid path issues
    tonic_build::configure()
        .build_server(true)
        .build_client(false) // We only need the server side
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", "::prost_types")
        .extern_path(".google.rpc.Status", "crate::bazel::rpc_status::Status")
        .compile_protos(
            &[
                "proto/bazel/remote_execution.proto",
                "proto/google/bytestream/bytestream.proto",
            ],
            &["proto"],
        )?;

    Ok(())
}
