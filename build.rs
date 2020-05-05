fn main() -> Result<(), std::io::Error> {
    tonic_build::configure()
        .build_server(false)
        .compile(
            &["dapr/proto/common/v1/common.proto", "dapr/proto/dapr/v1/dapr.proto"],
            &["."],
        )?;
   Ok(())
}
