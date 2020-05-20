// use std::env;

fn main() -> Result<(), std::io::Error> {
    // env::set_var("OUT_DIR", "src");
    tonic_build::configure().build_server(false).compile(
        &[
            "dapr/proto/common/v1/common.proto",
            "dapr/proto/dapr/v1/dapr.proto",
        ],
        &["."],
    )?;
    Ok(())
}
