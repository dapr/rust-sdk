use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let root_dir = {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir.parent().unwrap().to_path_buf()
    };

    // Gather relevant proto files: common + every file in runtime/v1
    let mut protos: Vec<PathBuf> = Vec::new();

    protos.push(root_dir.join("proto/dapr/proto/common/v1/common.proto"));

    let runtime_dir = root_dir.join("proto/dapr/proto/runtime/v1");
    let read_dir = fs::read_dir(&runtime_dir).expect("Failed to read runtime v1 proto directory");
    for entry in read_dir {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("proto") {
            protos.push(path);
        }
    }

    protos.sort_by_key(|p| p.to_string_lossy().to_string());

    let includes = vec![
        root_dir.join("proto"),
        root_dir.join("proto/dapr/proto/common/v1"),
        root_dir.join("proto/dapr/proto/runtime/v1"),
    ];

    // dapr
    proto_gen(
        root_dir.clone(),
        true,
        true,
        "dapr/src/dapr",
        &protos,
        &includes,
    );

    // example - helloworld
    proto_gen(
        root_dir.clone(),
        true,
        true,
        "examples/src/invoke/protos/",
        &[
            root_dir.join("examples/proto/helloworld/helloworld.proto"),
        ],
        &[
            root_dir.join("examples/proto/helloworld"),
        ],
    );
}

fn proto_gen(
    root_dir: PathBuf,
    build_client: bool,
    build_server: bool,
    out_dir: &str,
    protos: &[PathBuf],
    include_dirs: &[PathBuf],
) {
    let protos_display = protos
        .iter()
        .map(|p| p.strip_prefix(&root_dir).unwrap_or(p).to_string_lossy().to_string())
        .collect::<Vec<_>>();
    println!("protos: {protos_display:?}");

    let includes_display = include_dirs
        .iter()
        .map(|p| p.strip_prefix(&root_dir).unwrap_or(p).to_string_lossy().to_string())
        .collect::<Vec<_>>();
    println!("includes {includes_display:?}");

    let out_dir = root_dir.join(out_dir);
    println!("outdir {out_dir:?}");

    tonic_build::configure()
        .build_client(build_client)
        .build_server(build_server)
        .build_transport(true)
        .out_dir(out_dir.clone())
        .file_descriptor_set_path(out_dir.clone().join("types.bin"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(protos, include_dirs)
        .expect("Failed to compile protos");
}
