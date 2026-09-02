fn main() {
    #[cfg(feature = "desktop")]
    tauri_build::build();

    // screencapturekit's Swift bridge links `@rpath/libswift_Concurrency.dylib`.
    // That crate's `cargo:rustc-link-arg` rpath does not propagate to this
    // package's test/bin link line, so bake the Xcode Swift rpath here.
    #[cfg(target_os = "macos")]
    add_swift_concurrency_rpath();

    #[cfg(target_os = "macos")]
    compile_apple_speech();

    #[cfg(target_os = "macos")]
    compile_fn_tap();
}

#[cfg(target_os = "macos")]
fn add_swift_concurrency_rpath() {
    use std::process::Command;

    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    let Ok(output) = Command::new("xcode-select").arg("-p").output() else {
        return;
    };
    if !output.status.success() {
        return;
    }
    let xcode = String::from_utf8_lossy(&output.stdout).trim().to_string();
    for suffix in [
        "Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.5/macosx",
        "Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx",
    ] {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{xcode}/{suffix}");
    }
}

#[cfg(target_os = "macos")]
fn compile_apple_speech() {
    use std::env;
    use std::process::Command;

    let manifest = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let src = format!("{manifest}/native/apple_speech.swift");
    println!("cargo:rerun-if-changed={src}");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    let obj = format!("{out_dir}/apple_speech.o");
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "aarch64".into());
    let swift_arch = match arch.as_str() {
        "aarch64" => "arm64",
        other => other,
    };
    let target = format!("{swift_arch}-apple-macosx26.0");
    let status = Command::new("swiftc")
        .args([
            "-emit-object",
            "-parse-as-library",
            "-module-name",
            "SottoAppleSpeech",
            "-target",
            &target,
            "-o",
            &obj,
            &src,
            "-framework",
            "Speech",
            "-framework",
            "AVFoundation",
            "-framework",
            "Foundation",
        ])
        .status()
        .expect("swiftc apple_speech");
    if !status.success() {
        panic!("swiftc failed to compile native/apple_speech.swift");
    }
    println!("cargo:rustc-link-arg={obj}");
    println!("cargo:rustc-link-lib=framework=Speech");
    println!("cargo:rustc-link-lib=framework=AVFoundation");
    println!("cargo:rustc-link-lib=framework=Foundation");
}

#[cfg(target_os = "macos")]
fn compile_fn_tap() {
    use std::env;
    use std::process::Command;

    let manifest = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let src = format!("{manifest}/native/fn_tap.swift");
    println!("cargo:rerun-if-changed={src}");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    let obj = format!("{out_dir}/fn_tap.o");
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "aarch64".into());
    let swift_arch = match arch.as_str() {
        "aarch64" => "arm64",
        other => other,
    };
    let target = format!("{swift_arch}-apple-macosx26.0");
    let status = Command::new("swiftc")
        .args([
            "-emit-object",
            "-parse-as-library",
            "-module-name",
            "SottoFnTap",
            "-target",
            &target,
            "-o",
            &obj,
            &src,
            "-framework",
            "Cocoa",
            "-framework",
            "ApplicationServices",
            "-framework",
            "CoreGraphics",
        ])
        .status()
        .expect("swiftc fn_tap");
    if !status.success() {
        panic!("swiftc failed to compile native/fn_tap.swift");
    }
    println!("cargo:rustc-link-arg={obj}");
    println!("cargo:rustc-link-lib=framework=Cocoa");
    println!("cargo:rustc-link-lib=framework=ApplicationServices");
    println!("cargo:rustc-link-lib=framework=CoreGraphics");
}
