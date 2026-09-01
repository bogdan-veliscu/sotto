fn main() {
    #[cfg(feature = "desktop")]
    tauri_build::build();

    // screencapturekit's Swift bridge links `@rpath/libswift_Concurrency.dylib`.
    // That crate's `cargo:rustc-link-arg` rpath does not propagate to this
    // package's test/bin link line, so bake the Xcode Swift rpath here.
    #[cfg(target_os = "macos")]
    add_swift_concurrency_rpath();
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
