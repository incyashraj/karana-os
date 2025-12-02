# Karana OS: Hardware Roadmap & Reference Architecture

To achieve a "State of the Art" visionary device where the sky is no limit, we must decouple the **Compute** from the **Display**. Current standalone glasses are too weak. We will build a **Split-Architecture System**.

## 1. The Architecture: "The Sovereign Stack"

*   **The Head (Display Node)**: Dumb, high-fidelity display glasses. No heavy processing on the face (keeps it cool and light).
*   **The Core (Compute Node)**: A powerful, battery-operated unit worn on the belt or in a pocket. Runs Karana OS, local AI models, and renders the UI.
*   **The Link**: USB-C (Video + Data + Power).

## 2. Recommended Hardware (The "China" Dev Kit)

For the best balance of hackability, visual fidelity, and availability, I recommend the following stack:

### A. The Smart Glasses (Display)
**Recommendation: XREAL Air 2 Pro** or **Rokid Max**
*   **Why**: These are essentially 1080p Micro-OLED monitors that sit on your nose.
*   **Connection**: USB-C DisplayPort Alt Mode.
*   **OS Compatibility**: They appear as a standard HDMI/DP monitor to Linux. `karana-shell` will render directly to them without complex drivers.
*   **Sensors**: They have internal IMUs (Gyro/Accel). *Note: Accessing this raw data on Linux requires reverse-engineered drivers (OpenXR/Monado), but for a static HUD, they work out of the box.*

### B. The Compute Unit (The Brain)
**Recommendation: Orange Pi 5 Plus (16GB/32GB RAM)** or **Khadas Edge 2**
*   **Chipset**: Rockchip RK3588.
*   **Why**:
    *   **8-Core CPU**: Fast enough for a smooth UI.
    *   **NPU (6 TOPS)**: Critical for running local AI (Vision/LLM) without cloud lag.
    *   **Video Output**: Supports USB-C DisplayPort (direct connection to glasses).
    *   **Linux Support**: Excellent mainline Linux support (Armbian/Ubuntu).

### C. Peripherals
*   **Input**: "Rii i4" Mini Bluetooth Keyboard with Trackpad (handheld controller).
*   **Power**: Any 65W PD Power Bank (Anker/Baseus) to power both the Pi and the Glasses.
*   **Camera**: The Orange Pi supports MIPI cameras. You can mount a small camera module on the glasses frame for Computer Vision (Object detection).

## 3. Development & Testing Strategy

### Phase 1: The "Tethered" Simulator (Current)
*   **Hardware**: Your Laptop + XREAL Glasses.
*   **Setup**: Plug glasses into laptop. Set as "External Monitor".
*   **Test**: Run `karana-shell`. Move the window to the glasses.
*   **Goal**: Perfect the UI layout, color contrast (pure black = transparent), and text readability.

### Phase 2: The "Portable" Prototype
*   **Hardware**: Orange Pi 5 + Power Bank + Glasses.
*   **Setup**: Install Linux (Armbian) on Orange Pi. Clone `karana-os`. Build `karana-core`.
*   **Test**: Walk around. Test battery life. Test "Sovereign Mode" (offline AI).
*   **Goal**: System integration and power management.

### Phase 3: The "Visionary" Custom Build
*   **Hardware**: Custom PCB (Carrier board for RK3588 Compute Module) + Waveguide Optics.
*   **Goal**: Shrink the Orange Pi into a custom 3D-printed "Puck" case. Integrate a high-quality camera for the "AI Vision" features.

## 4. Shopping List (AliExpress / Amazon)

1.  **XREAL Air 2 Pro**: ~$450 (Best transparency control).
2.  **Orange Pi 5 (16GB)**: ~$140 (Need RAM for AI).
3.  **USB-C Data/Video Cable**: High quality, flexible.
4.  **Rii i4 Mini Keyboard**: ~$25.

## 5. Why this path?
Building your own optics from scratch is a multi-million dollar trap. By using XREAL/Rokid as a "dumb display," you leverage millions of dollars of R&D in optics, while you focus on the **Software Magic (Karana OS)** and the **Compute Power**. This is how you build a device that feels "State of the Art" today.
