fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile XCBBuildService proto files
    tonic_build::configure()
        .build_server(true)
        .build_client(false) // We only need the server side
        .compile_protos(
            &["proto/xcode/cas.proto", "proto/xcode/keyvalue.proto"],
            &["proto/xcode"],
        )?;

    Ok(())
}
