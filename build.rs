use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Register custom cfg flags to avoid warnings
    println!("cargo::rustc-check-cfg=cfg(gpu_available)");
    println!("cargo::rustc-check-cfg=cfg(metal_gpu_available)");
    println!("cargo::rustc-check-cfg=cfg(nvidia_gpu_available)");
    println!("cargo::rustc-check-cfg=cfg(intel_mkl_available)");
    println!("cargo::rustc-check-cfg=cfg(gpu_hardware_detected)");
    
    // Detect available GPU capabilities and set cfg flags
    detect_and_set_gpu_flags();
    
    // Print helpful build information
    print_build_info();
}

fn detect_and_set_gpu_flags() {
    // Set platform-specific GPU availability flags
    if cfg!(target_os = "macos") && has_metal_support() {
        println!("cargo:rustc-cfg=metal_gpu_available");
        eprintln!("🍎 Build: Metal GPU detected on macOS");
    }
    
    if has_nvidia_gpu() {
        println!("cargo:rustc-cfg=nvidia_gpu_available");
        if cfg!(target_os = "linux") {
            eprintln!("🟢 Build: NVIDIA GPU detected on Linux");
        } else if cfg!(target_os = "windows") {
            eprintln!("🟢 Build: NVIDIA GPU detected on Windows");
        }
    }
    
    if has_intel_mkl() {
        println!("cargo:rustc-cfg=intel_mkl_available");
        eprintln!("🔵 Build: Intel MKL detected");
    }
    
    // Set general GPU availability flag
    if has_any_gpu() {
        println!("cargo:rustc-cfg=gpu_hardware_detected");
    }
}

fn print_build_info() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".to_string());
    eprintln!("🔧 Build: Target OS: {}", target_os);
    
    // Check which features are enabled
    let mut enabled_features = Vec::new();
    
    if cfg!(feature = "llama") {
        enabled_features.push("Llama");
    }
    if cfg!(feature = "parallel") {
        enabled_features.push("Parallel");
    }
    if cfg!(feature = "gpu-metal") {
        enabled_features.push("Metal GPU");
    }
    if cfg!(feature = "gpu-cuda") {
        enabled_features.push("CUDA GPU");
    }
    if cfg!(feature = "gpu-mkl") {
        enabled_features.push("Intel MKL");
    }
    
    eprintln!("🚀 Build: Enabled features: {}", 
        if enabled_features.is_empty() { 
            "CPU-only".to_string() 
        } else { 
            enabled_features.join(", ") 
        }
    );
    
    // Provide recommendations if auto-gpu is enabled but no GPU features are active
    if cfg!(feature = "auto-gpu") && !cfg!(any(feature = "gpu-metal", feature = "gpu-cuda", feature = "gpu-mkl")) {
        print_gpu_recommendations(&target_os);
    }
}

fn print_gpu_recommendations(target_os: &str) {
    eprintln!("💡 Build: Auto-GPU enabled but no GPU features active.");
    
    match target_os {
        "macos" => {
            if has_metal_support() {
                eprintln!("   🍎 Recommendation: Add --features gpu-metal for Metal acceleration");
            } else {
                eprintln!("   💻 Metal not available on this macOS system");
            }
        }
        "linux" => {
            if has_nvidia_gpu() {
                eprintln!("   🟢 Recommendation: Add --features gpu-cuda for NVIDIA GPU acceleration");
            } else if has_intel_mkl() {
                eprintln!("   🔵 Recommendation: Add --features gpu-mkl for Intel MKL acceleration");
            } else {
                eprintln!("   💻 No GPU acceleration detected on this Linux system");
            }
        }
        "windows" => {
            if has_nvidia_gpu() {
                eprintln!("   🟢 Recommendation: Add --features gpu-cuda for NVIDIA GPU acceleration");
            } else if has_intel_mkl() {
                eprintln!("   🔵 Recommendation: Add --features gpu-mkl for Intel MKL acceleration");
            } else {
                eprintln!("   💻 No GPU acceleration detected on this Windows system");
            }
        }
        _ => {
            eprintln!("   💻 GPU detection not implemented for target OS: {}", target_os);
        }
    }
}

fn has_metal_support() -> bool {
    #[cfg(target_os = "macos")]
    {
        // Metal is available on all modern Macs (macOS 10.11+)
        match Command::new("system_profiler").args(&["SPDisplaysDataType"]).output() {
            Ok(output) => {
                let info = String::from_utf8_lossy(&output.stdout);
                info.contains("Metal") || info.contains("GPU")
            }
            Err(_) => true, // Assume Metal is available on macOS if system_profiler fails
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

fn has_nvidia_gpu() -> bool {
    // Check for NVIDIA GPU across platforms
    let nvidia_commands = [
        ("nvidia-smi", vec!["--list-gpus"]),
        ("nvidia-smi.exe", vec!["--list-gpus"]),
        ("nvcc", vec!["--version"]),
        ("nvcc.exe", vec!["--version"]),
    ];
    
    for (cmd, args) in &nvidia_commands {
        if Command::new(cmd).args(args).output().is_ok() {
            return true;
        }
    }
    
    // Check environment variables
    env::var("CUDA_PATH").is_ok() 
        || env::var("CUDA_HOME").is_ok() 
        || env::var("NVIDIA_VISIBLE_DEVICES").is_ok()
}

fn has_intel_mkl() -> bool {
    // Check for Intel MKL
    if env::var("MKLROOT").is_ok() || env::var("MKL_ROOT").is_ok() {
        return true;
    }
    
    // Check for Intel oneAPI (simplified paths)
    let oneapi_paths = if cfg!(target_os = "windows") {
        vec![
            "C:/Program Files (x86)/Intel/oneAPI/mkl",
            "C:/Program Files/Intel/oneAPI/mkl",
        ]
    } else {
        vec![
            "/opt/intel/oneapi/mkl",
            "/usr/local/intel/oneapi/mkl",
        ]
    };
    
    oneapi_paths.iter().any(|path| std::path::Path::new(path).exists())
}

fn has_any_gpu() -> bool {
    has_metal_support() || has_nvidia_gpu() || has_intel_mkl()
}
