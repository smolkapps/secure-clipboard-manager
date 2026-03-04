fn main() {
    println!("cargo:rustc-link-lib=framework=ServiceManagement");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=AppKit");
}
