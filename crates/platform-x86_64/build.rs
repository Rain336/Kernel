fn main() {
    println!("cargo:rustc-link-arg=-Tcrates/platform-x86_64/linker.ld");
    println!("cargo:rerun-if-changed=linker.ld");
}
