fn main() {
    // this code assumes that directory and package name of the shared crate are the same
    relib_interface::module::generate(
        "../shared/src/exports.rs",
        "shared::exports::Exports",
        "../shared/src/imports.rs",
        "shared::imports::Imports",
    );
}
