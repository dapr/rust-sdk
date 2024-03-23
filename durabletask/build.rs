use prost_build::Config;

fn main() -> Result<(), std::io::Error> {
    let mut config = Config::default();
    config.default_package_filename("durabletask");
    tonic_build::configure()
        .build_server(false)
        .compile_with_config(
            config,
            &["submodules/durabletask-protobuf/protos/orchestrator_service.proto"],
            &["."],
        )?;
    Ok(())
}
