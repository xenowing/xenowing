fn main() {
    println!("cargo:rerun-if-changed=src/_cycles.s");
    println!("cargo:rerun-if-changed=src/entry.s");
}
