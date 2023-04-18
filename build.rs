fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(false)
        //.protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["proto/splitwiser.proto"], &["proto/"])
        .map_err(Into::into)
}
