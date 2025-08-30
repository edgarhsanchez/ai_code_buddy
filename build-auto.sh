#!/bin/bash
# Auto-build script for AI Code Buddy with optimal GPU features

set -e

echo "🚀 AI Code Buddy Auto-Build Script"
echo "=================================="

# Detect the operating system
OS=$(uname -s)
echo "🔧 Detected OS: $OS"

# Default features
FEATURES="llama,parallel"

# Platform-specific GPU feature detection
case "$OS" in
    "Darwin")  # macOS
        echo "🍎 macOS detected - checking for Metal support..."
        if system_profiler SPDisplaysDataType 2>/dev/null | grep -q "Metal"; then
            FEATURES="${FEATURES},gpu-metal"
            echo "✅ Metal GPU support detected and enabled"
        else
            echo "💻 No Metal support detected - using CPU only"
        fi
        ;;
    "Linux")
        echo "🐧 Linux detected - checking for GPU support..."
        if command -v nvidia-smi >/dev/null 2>&1; then
            FEATURES="${FEATURES},gpu-cuda"
            echo "✅ NVIDIA GPU detected - CUDA support enabled"
        elif [ -d "/opt/intel/oneapi/mkl" ] || [ -n "$MKLROOT" ]; then
            FEATURES="${FEATURES},gpu-mkl"
            echo "✅ Intel MKL detected - MKL support enabled"
        else
            echo "💻 No GPU acceleration detected - using CPU only"
        fi
        ;;
    "CYGWIN"*|"MINGW"*|"MSYS"*)  # Windows
        echo "🪟 Windows detected - checking for GPU support..."
        if command -v nvidia-smi.exe >/dev/null 2>&1; then
            FEATURES="${FEATURES},gpu-cuda"
            echo "✅ NVIDIA GPU detected - CUDA support enabled"
        elif [ -d "/c/Program Files/Intel/oneAPI/mkl" ] || [ -d "/c/Program Files (x86)/Intel/oneAPI/mkl" ]; then
            FEATURES="${FEATURES},gpu-mkl"
            echo "✅ Intel MKL detected - MKL support enabled"
        else
            echo "💻 No GPU acceleration detected - using CPU only"
        fi
        ;;
    *)
        echo "❓ Unknown OS - using CPU-only build"
        ;;
esac

echo ""
echo "🎯 Building with features: $FEATURES"
echo ""

# Determine build type
BUILD_TYPE="debug"
if [ "$1" = "--release" ]; then
    BUILD_TYPE="release"
    echo "🏗️  Building in release mode for optimal performance..."
    cargo build --release --features "$FEATURES"
else
    echo "🏗️  Building in debug mode (use --release for production)..."
    cargo build --features "$FEATURES"
fi

echo ""
echo "✅ Build completed successfully!"
echo ""
echo "📖 Usage examples:"
echo "  ./target/$BUILD_TYPE/ai-code-buddy --cli"
echo "  ./target/$BUILD_TYPE/ai-code-buddy --cli --parallel"
echo "  ./target/$BUILD_TYPE/ai-code-buddy --tui"
echo ""
echo "🔧 To install system-wide:"
echo "  cargo install --path . --features \"$FEATURES\""
