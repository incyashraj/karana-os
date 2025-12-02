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
