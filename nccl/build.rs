﻿fn main() {
    use std::{env, path::PathBuf, process::Command};

    let Some(cuda_root) = find_cuda_helper::find_cuda_root() else {
        return;
    };
    let Ok(output) = Command::new("ldconfig").arg("-p").output() else {
        return;
    };
    if !unsafe { String::from_utf8_unchecked(output.stdout) }.contains("nccl") {
        return;
    }

    println!("cargo:rustc-cfg=detected_nccl");

    // Tell cargo to tell rustc to link the cuda library.
    find_cuda_helper::include_cuda();
    println!("cargo:rustc-link-lib=dylib=nccl");

    // Tell cargo to invalidate the built crate whenever the wrapper changes.
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point to bindgen,
    // and lets you build up options for the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate bindings for.
        .header("wrapper.h")
        .clang_arg(format!("-I{}/include", cuda_root.display()))
        // Only generate bindings for the functions in these namespaces.
        .allowlist_function("nccl.*")
        .allowlist_item("nccl.*")
        // Annotate the given type with the #[must_use] attribute.
        .must_use_type("ncclResult_t")
        // Generate rust style enums.
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
        // Use core instead of std in the generated bindings.
        .use_core()
        // Tell cargo to invalidate the built crate whenever any of the included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
