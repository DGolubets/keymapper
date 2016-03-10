use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("../../../");
    let res_path = Path::new("resources");

    fs::create_dir(dest_path.join("resources"));
    fs::copy(res_path.join("log.toml"), dest_path.join("resources/log.toml"));
    fs::copy(res_path.join("application.conf"), dest_path.join("resources/application.conf"));
}
