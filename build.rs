use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo::rustc-check-cfg=cfg(gpu_available)");

    // Detect GPU capabilities and automatically enable features
    let detected_features = detect_gpu_capabilities();
    let has_gpu = !detected_features.is_empty();

    // If we detected GPU capabilities, enable them by default
    for feature in detected_features {
        println!("cargo:rustc-cfg=feature=\"{feature}\"");
        eprintln!("🔧 Build: Auto-detected GPU feature: {feature}");
    }

    // Set default GPU mode based on detection
    if has_gpu {
        println!("cargo:rustc-cfg=gpu_available");
        eprintln!("🚀 Build: GPU acceleration will be enabled by default");
    } else {
        eprintln!("💻 Build: No GPU acceleration detected - CPU mode will be used");
    }
}

fn detect_gpu_capabilities() -> Vec<String> {
    let mut features = Vec::new();

    // Detect Apple Silicon (Metal) — only on macOS
    if cfg!(target_os = "macos") && is_apple_silicon() {
        features.push("gpu-metal".to_string());
        eprintln!("🍎 Build: Apple Silicon detected - enabling Metal GPU support");
    }

    // Detect NVIDIA GPU (CUDA) — only enable on Windows builds per policy
    if cfg!(target_os = "windows") && has_nvidia_gpu() {
        features.push("gpu-cuda".to_string());
        eprintln!("🟢 Build: NVIDIA GPU detected - enabling CUDA support");
    }

    // Detect Intel MKL — keep generic, but it won't pull CUDA
    if has_intel_mkl() {
        features.push("gpu-mkl".to_string());
        eprintln!("🔵 Build: Intel MKL detected - enabling MKL support");
    }

    features
}

fn is_apple_silicon() -> bool {
    #[cfg(target_os = "macos")]
    {
        // Check if running on Apple Silicon
        match Command::new("uname").arg("-m").output() {
            Ok(output) => {
                let arch = String::from_utf8_lossy(&output.stdout);
                arch.trim() == "arm64"
            }
            Err(_) => false,
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

fn has_nvidia_gpu() -> bool {
    // Try to detect NVIDIA GPU
    #[cfg(target_os = "linux")]
    {
        // Check for nvidia-smi
        if Command::new("nvidia-smi").output().is_ok() {
            return true;
        }

        // Check for CUDA runtime
        if Command::new("nvcc").arg("--version").output().is_ok() {
            return true;
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Check for NVIDIA driver
        if Command::new("nvidia-smi.exe").output().is_ok() {
            return true;
        }
    }

    // Check environment variables
    env::var("CUDA_PATH").is_ok() || env::var("CUDA_HOME").is_ok()
}

fn has_intel_mkl() -> bool {
    // Check for Intel MKL
    env::var("MKLROOT").is_ok()
        || env::var("MKL_ROOT").is_ok()
        || Command::new("pkg-config")
            .args(["--exists", "mkl"])
            .output()
            .is_ok_and(|o| o.status.success())
}
