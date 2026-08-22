fn main() {
    println!("cargo:rerun-if-changed=../web/dist");

    // Ensure ../web/dist and a fallback index.html exist so rust-embed
    // doesn't fail compilation if the frontend hasn't been built yet.
    let dist_dir = std::path::Path::new("../web/dist");
    if !dist_dir.exists() {
        let _ = std::fs::create_dir_all(dist_dir);
    }
    let placeholder = dist_dir.join("index.html");
    if !placeholder.exists() {
        let _ = std::fs::write(
            placeholder,
            b"<!DOCTYPE html><html><body><p>Frontend not built. Run 'pnpm run build' in web/.</p></body></html>",
        );
    }
}
