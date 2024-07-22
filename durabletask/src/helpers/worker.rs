use gethostname::gethostname;

pub fn get_default_worker_name() -> String {
    let hostname = gethostname();
    // Had to make two line since I got a freed value error
    let hostname = hostname.to_str().unwrap_or("unknown");
    let pid = std::process::id();
    let uuid = uuid::Uuid::new_v4().to_string();
    format!("{hostname},{pid},{uuid}")
}
