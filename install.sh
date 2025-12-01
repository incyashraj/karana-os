#!/bin/bash
set -e

echo "🔮 Installing Kāraṇa OS v1.0..."

# 1. Check prerequisites
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 2. Build Core
echo "⚙️  Building Core System (Release Mode)..."
if [ -d "karana-core" ]; then
    cd karana-core
    cargo build --release
    cd ..
else
    echo "❌ Error: karana-core directory not found."
    exit 1
fi

# 3. Setup Runtime Environment
echo "📂 Setting up Runtime State..."
# Create the persistence layer structure
DIRS=(
    "karana-cache"
    "karana-governance"
    "karana-ledger"
)

for dir in "${DIRS[@]}"; do
    if [ ! -d "$dir" ]; then
        echo "   Creating $dir..."
        mkdir -p "$dir"
    fi
done

# 4. Create Launcher
echo "🚀 Creating Launcher Script..."
cat > start_karana.sh << 'EOF'
#!/bin/bash
# Kāraṇa OS Launcher
# Ensures the environment is correctly set up before running the core

# Get the absolute path of the installation directory
INSTALL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "🔮 Booting Kāraṇa OS..."
cd "$INSTALL_DIR"

# Check if binary exists
if [ ! -f "./karana-core/target/release/karana-core" ]; then
    echo "❌ Binary not found. Please run ./install.sh first."
    exit 1
fi

# Run the core
./karana-core/target/release/karana-core
EOF

chmod +x start_karana.sh

echo "✅ Installation Complete!"
echo "---------------------------------------------------"
echo "To start the OS, run:"
echo "  ./start_karana.sh"
echo "---------------------------------------------------"
