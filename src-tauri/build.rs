fn main() {
    println!("cargo:rerun-if-changed=src");
    tauri_rs_ts_ipc::build();
    tauri_build::build()
}
