use std::env;
use std::path::Path;

fn main() {
    // Slint UIコンパイル
    if Path::new("ui/main_window.slint").exists() {
        slint_build::compile("ui/main_window.slint").expect("Slint build failed");
    } else {
        // 開発初期段階ではファイルが存在しない可能性がある
        println!("cargo:warning=Slint UI file not found, skipping UI compilation");
    }

    // プラットフォーム固有の設定
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    match target_os.as_str() {
        "windows" => {
            println!("cargo:rustc-link-lib=winmm");
            println!("cargo:rustc-link-lib=dsound");
            println!("cargo:rustc-link-lib=ole32");
        }
        "macos" => {
            println!("cargo:rustc-link-lib=framework=CoreAudio");
            println!("cargo:rustc-link-lib=framework=AudioUnit");
            println!("cargo:rustc-link-lib=framework=AudioToolbox");
        }
        "linux" => {
            println!("cargo:rustc-link-lib=alsa");
            println!("cargo:rustc-link-lib=pulse");
            println!("cargo:rustc-link-lib=pulse-simple");
        }
        _ => {}
    }

    // 環境変数の設定
    println!("cargo:rerun-if-changed=ui/");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");

    // デバッグ情報
    if env::var("PROFILE").unwrap() == "debug" {
        println!("cargo:rustc-cfg=debug_build");
    }
}
