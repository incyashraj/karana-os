# Kāraṇa OS: AI Strategy (Hybrid Intelligence)

> **Problem**: Smart Glasses and IoT devices have limited storage (<32GB) and RAM (<2GB). Large Language Models (LLMs) like Llama-3 (8GB+) are too big.
> **Solution**: A Hybrid "Download-on-Demand" architecture using Quantized Small Language Models (SLMs).

## 🧠 The "Reflex" Brain: TinyLlama 1.1B (Quantized)

We use **TinyLlama-1.1B-Chat**, quantized to **4-bit (Q4_K_M)**.
*   **Size**: ~670 MB (Fits easily on 8GB SD cards).
*   **RAM Usage**: ~800 MB (Fits on Raspberry Pi Zero 2 W / 4).
*   **Speed**: Real-time on ARM CPUs (via `candle` SIMD optimization).

## 📦 Download-on-Demand

To keep the OS image small (<100MB) and git-friendly:
1.  **Core OS**: Contains only the *code* to run AI (`candle` inference engine).
2.  **Simulation Mode**: Out-of-the-box, the OS uses a rule-based simulator (0MB).
3.  **Ignition**: The user runs `install ai-core` (or `karana-core --install-ai`).
4.  **Fetch**: The OS downloads the `.gguf` model from HuggingFace to `karana-cache/models/`.

## 🛠️ Implementation Details

### 1. Model Manager (`src/ai/mod.rs`)
*   Checks `karana-cache/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf`.
*   If missing -> Falls back to `predict_simulated()`.
*   If present -> Loads into RAM using `candle_transformers::models::quantized_llama`.

### 2. Inference Pipeline
*   **Tokenizer**: `TinyLlama/TinyLlama-1.1B-Chat-v1.0` (tokenizer.json).
*   **Format**: `<|system|>...</s><|user|>...</s><|assistant|>`
*   **Engine**: CPU-based inference (optimized for ARM NEON/AVX).

## 🚀 Future Roadmap (Phase 3)
*   **NPU Acceleration**: Use `candle-metal` (Apple) or `candle-wgpu` (Vulkan) for hardware acceleration on glasses.
*   **Swarm Offloading**: If the intent is too complex for TinyLlama (e.g., "Summarize this 50-page PDF"), the OS will ZK-encrypt the prompt and send it to a powerful peer in the swarm.
