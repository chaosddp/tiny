use std::{env, fs, path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=builtin/init.lua");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = &Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("builtin");

    if !dest_path.exists() {
        fs::create_dir(dest_path).unwrap();
    }

    Command::new("cp")
        .args(["-r", "builtin/*", dest_path.to_str().unwrap()])
        .spawn()
        .expect("fail to copy builtin file");
}
