fn main() {
    // Development also embeds assets; frontend edits must invalidate Rust's cache.
    println!("cargo:rerun-if-changed=../dist");
    tauri_build::build();
}
