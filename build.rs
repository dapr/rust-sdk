// use std::env;

fn main() -> Result<(), std::io::Error> {
    // env::set_var("OUT_DIR", "src");
    tonic_build::configure().build_server(true).compile(
        &[
            "dapr/proto/common/v1/common.proto",
            "dapr/proto/runtime/v1/dapr.proto",
            "dapr/proto/runtime/v1/appcallback.proto",
            "examples/invoke/proto/helloworld.proto",
        ],
        &["."],
    )?;
    Ok(())
}
