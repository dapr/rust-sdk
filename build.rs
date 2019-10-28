fn main() -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir("proto")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            panic!("Subdirectories in the proto directory are not currently supported")
        }

        tonic_build::compile_protos(path).unwrap();
    }
    Ok(())
}
