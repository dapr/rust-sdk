use std::env;

use std::path::PathBuf;

fn main() {
    let root_dir = {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir.parent().unwrap().to_path_buf()
    };

    // dapr
    // env::set_var("OUT_DIR", "src");
    proto_gen(
        root_dir.clone(),
        true,
        true,
        "dapr/src/dapr",
        &[
            "proto/dapr/proto/common/v1/common.proto",
            "proto/dapr/proto/runtime/v1/dapr.proto",
            "proto/dapr/proto/runtime/v1/appcallback.proto",
        ],
        &[
            "proto",
            "proto/dapr/proto/common/v1",
            "proto/dapr/proto/runtime/v1",
        ],
    );

    // example - helloworld
    proto_gen(
        root_dir.clone(),
        true,
        true,
        "examples/src/invoke/protos/",
        &["examples/proto/helloworld/helloworld.proto"],
        &["examples/proto/helloworld"],
    );
}

fn proto_gen(
    root_dir: PathBuf,
    build_client: bool,
    build_server: bool,
    out_dir: &str,
    include_dirs: &[&str],
    interface: &[&str],
) {
    let include_dirs = include_dirs
        .iter()
        .map(|path| format!("{}/{}", root_dir.to_str().unwrap(), path))
        .collect::<Vec<_>>();

    println!("included {include_dirs:?}");

    let interface = interface
        .iter()
        .map(|path| format!("{}/{}", root_dir.to_str().unwrap(), path))
        .collect::<Vec<_>>();
    println!("interface {interface:?}");

    let out_dir = root_dir.join(out_dir);
    println!("outdir {out_dir:?}");

    tonic_build::configure()
        .build_client(build_client)
        .build_server(build_server)
        .build_transport(true)
        .out_dir(out_dir.clone())
        .file_descriptor_set_path(out_dir.clone().join("types.bin"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&include_dirs, &interface)
        .expect("Failed to compile protos");
}
