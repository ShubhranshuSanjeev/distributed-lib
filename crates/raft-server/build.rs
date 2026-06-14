fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/raft/raft.proto");

    let fds = protox::Compiler::new(["../../proto"])?
        .include_source_info(true)
        // didn't want to regenerate types because
        // they are already available via the `raft-models` crate
        .include_imports(false)
        .open_files(["../../proto/raft/service/service.proto"])?
        .file_descriptor_set();

    tonic_prost_build::configure()
        .out_dir("generated")
        .build_client(false)
        .build_server(true)
        .compile_fds(fds)?;

    Ok(())
}
