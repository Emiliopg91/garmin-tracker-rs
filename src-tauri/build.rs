fn main() {
    println!("cargo:rerun-if-changed=src");
    if std::env::var("PROFILE") == Ok("debug".to_string()) {
        tauri_rs_ts_ipc::build();
    }
    tauri_build::build()
}
