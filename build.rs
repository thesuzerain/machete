fn main() -> Result<(), nlprule_build::Error> {
    println!("cargo:rerun-if-changed=build.rs");

    nlprule_build::BinaryBuilder::new(
        &["en"],
        std::env::var("OUT_DIR").expect("OUT_DIR is set when build.rs is running"),
    )
    .build()?
    .validate()
}
