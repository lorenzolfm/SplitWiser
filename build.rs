fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(false)
        .compile(&["proto/splitwiser.proto"], &["proto/"])
        .map_err(Into::into)
}
