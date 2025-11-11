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

    // Generate C header file using cbindgen
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let output_file = std::path::Path::new(&crate_dir)
        .join("include")
        .join("fabrik.h");

    // Create include directory if it doesn't exist
    if let Some(parent) = output_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let cbindgen_config = std::path::Path::new(&crate_dir).join("cbindgen.toml");
    cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(cbindgen::Config::from_file(&cbindgen_config)?)
        .generate()
        .map_err(|e| format!("Unable to generate bindings: {:?}", e))?
        .write_to_file(&output_file);

    println!("cargo:rerun-if-changed=src/capi/mod.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    Ok(())
}
