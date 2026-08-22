fn main() {
    // Invalidate build cache if frontend web assets are re-built
    println!("cargo:rerun-if-changed=../web/dist");
}
