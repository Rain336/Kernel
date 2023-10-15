fn main() {
    let script = std::env::current_dir().unwrap().join("linker.ld");

    println!("cargo:rustc-link-arg=-T{}", script.display());
    println!("cargo:rerun-if-changed=linker.ld");
}
