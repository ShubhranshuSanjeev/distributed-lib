fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/raft/raft.proto");

    // Using protobuf to manage all my types
    // Wanted to keep all type definitions at one place, along with
    // rpc stub definitions
    let fds = protox::compile(
        [
            "../../proto/raft/rpc-models/rpc-models.proto",
            "../../proto/raft/state/state.proto",
        ],
        ["../../proto"],
    )?;
    tonic_prost_build::configure()
        .out_dir("generated")
        .build_client(false)
        .build_server(false)
        .compile_fds(fds)?;

    Ok(())
}
