use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("../../../");
    let res_path = Path::new("resources");
    let target_resources_path = dest_path.join("resources");

    if target_resources_path.exists() {
        fs::remove_dir_all(&target_resources_path).expect("Failed to remove resource directory");
    }

    fs::create_dir(&target_resources_path).expect("Failed to create resource directory");

    let files_to_copy = vec!["log.toml", "application.conf", "profiles.xml"];

    for file_name in files_to_copy {
        fs::copy(res_path.join(file_name), target_resources_path.join(file_name)).unwrap();
    }
}
