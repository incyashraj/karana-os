# Kāraṇa OS Simulator - Desktop Testing Guide

## 🚀 Quick Start

### Prerequisites
- Node.js v18+ 
- Rust toolchain (for backend)
- Modern browser (Chrome/Firefox/Edge recommended)

### Start Development Server

```bash
# Terminal 1: Start Frontend
cd simulator-ui
npm install
npm run dev
# Opens at http://localhost:3000

# Terminal 2: Start Backend (optional)
cd karana-core
cargo run --bin karana_api_server
# Runs at http://localhost:8080
```

## 🎮 Testing Without Hardware

The simulator is designed to work fully on desktop browsers without AR glasses. All features are accessible via keyboard and mouse.

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl/Cmd + V` | Open Vision Analyzer |
| `Ctrl/Cmd + O` | Open Oracle Chat |
| `Ctrl/Cmd + W` | Open Wallet |
| `Ctrl/Cmd + A` | Open AR Workspace |
| `Ctrl/Cmd + ,` | Open Settings |
| `Ctrl/Cmd + T` | Toggle Timers |
| `Ctrl/Cmd + N` | Toggle Notifications |
| `Escape` | Return to Home |
| `?` | Show Keyboard Help |

## 🧪 Testing Checklist

### 1. Welcome Screen
- [ ] First-time load shows welcome modal
- [ ] "Initialize System" button dismisses and doesn't show again
- [ ] Can clear localStorage and see welcome again

### 2. Vision Analyzer Mode
- [ ] Camera permission request (or fallback image)
- [ ] Scanning animation plays
- [ ] Mock analysis displays results
- [ ] Confidence score visible
- [ ] Tags render correctly

### 3. Oracle Chat Mode
- [ ] Text input works
- [ ] Messages appear in chat history
- [ ] Intent detection shows (TRANSFER, TIMER, etc.)
- [ ] Processing state displays

### 4. Wallet Mode
- [ ] "Create Wallet" button works
- [ ] Recovery phrase modal shows 24 words
- [ ] Balance displays
- [ ] Transaction history renders
- [ ] Transaction signing modal appears

### 5. AR Workspace Mode
- [ ] Window spawning works (Video, Browser, Terminal)
- [ ] Windows are draggable
- [ ] Windows can be closed
- [ ] Multiple windows supported

### 6. Settings Panel
- [ ] All 4 tabs accessible (UX, Security, Privacy, System)
- [ ] UX Level changes apply
- [ ] Security Preset selectable
- [ ] Ephemeral Mode toggle works
- [ ] Privacy indicators update in HUD
- [ ] Click outside closes panel

### 7. Timers & Notifications
- [ ] Can create timer via Oracle ("set timer 5 minutes")
- [ ] Timer counts down
- [ ] Pause/Resume works
- [ ] Notification appears on completion
- [ ] Notification badges show count
- [ ] Dismiss notifications works

### 8. HUD (Heads-Up Display)
- [ ] Time updates every second
- [ ] Battery indicator visible
- [ ] Wallet balance updates
- [ ] Ephemeral mode badge turns purple
- [ ] Mode-specific color themes apply

### 9. Keyboard Shortcuts
- [ ] All shortcuts respond
- [ ] Help overlay shows all shortcuts
- [ ] No conflicts with browser shortcuts

### 10. Error Handling
- [ ] Backend disconnection banner shows
- [ ] Error boundary catches crashes
- [ ] Error toast dismissible
- [ ] Invalid inputs handled gracefully

## 🐛 Known Issues & Workarounds

### Issue: Camera Not Working
**Solution**: The simulator falls back to a placeholder image. This is expected behavior when camera permissions are denied.

### Issue: Backend Not Connected
**Solution**: The UI works in "mock mode" when backend is offline. Some features (real AI analysis, blockchain transactions) require the backend.

### Issue: Welcome Screen Keeps Showing
**Solution**: Check browser localStorage. Clear it with `localStorage.clear()` in DevTools if needed.

### Issue: Keyboard Shortcuts Not Working
**Solution**: Make sure focus is on the app window (click anywhere in the UI first).

## 🎨 UI/UX Features

### Glassmorphism Design
- Transparent panels with backdrop blur
- Cyberpunk aesthetic with neon accents
- Adaptive color themes per mode

### Accessibility
- Full keyboard navigation
- Focus indicators
- High contrast support
- Reduced motion support

### Responsive Design
- Desktop optimized (1920x1080+)
- Tablet compatible (768px+)
- Mobile support (experimental)

## 🔧 Developer Tools

### Chrome DevTools
- Open with `F12` or `Ctrl+Shift+I`
- **Console**: View logs and errors
- **Network**: Monitor API calls
- **Application > Local Storage**: Check persisted data

### React DevTools
Install React DevTools extension for component debugging.

### Performance Monitoring
- Check FPS with `Ctrl+Shift+P` → "Show FPS meter" (Chrome)
- Monitor memory in DevTools Performance tab

## 📊 Testing Scenarios

### Scenario 1: New User Onboarding
1. Clear localStorage
2. Refresh page
3. Welcome screen should appear
4. Click "Initialize System"
5. Verify home screen with 3 mode buttons

### Scenario 2: Vision Analysis Workflow
1. Press `Ctrl+V` or click "ANALYZE"
2. Allow camera (or see fallback)
3. Wait for scan animation
4. Verify mock results display
5. Press `Esc` to return home

### Scenario 3: Oracle Command Flow
1. Press `Ctrl+O` or click "ORACLE"
2. Type: "transfer 100 tokens to did:karana:alice"
3. Submit
4. Verify intent detected as TRANSFER
5. Transaction modal should appear
6. Sign or cancel

### Scenario 4: Settings Configuration
1. Press `Ctrl+,` or click "SYSTEM"
2. Navigate through all 4 tabs
3. Change UX Level to "EXPERT"
4. Enable Ephemeral Mode
5. Verify HUD badge turns purple
6. Close settings

### Scenario 5: Timer Creation
1. Press `Ctrl+O` for Oracle
2. Say: "set timer 2 minutes for coffee"
3. Verify timer appears in timers panel
4. Press `Ctrl+T` to toggle timers view
5. Watch countdown
6. Verify notification on completion

## 🚨 Troubleshooting

### App Won't Load
1. Check console for errors
2. Verify `npm run dev` is running
3. Try clearing browser cache
4. Check for port conflicts (default: 3000)

### Styles Not Applying
1. Hard refresh: `Ctrl+Shift+R`
2. Verify `styles.css` loaded in Network tab
3. Check for Tailwind CDN connection

### Features Not Working
1. Check if backend is required (see banner)
2. Verify localStorage isn't full
3. Test in incognito mode
4. Try different browser

## 📝 Reporting Issues

When reporting bugs, include:
- Browser version
- Console errors (F12 → Console)
- Steps to reproduce
- Screenshot/video if visual bug

## 🎯 Next Steps

Once all features test successfully:
1. ✅ Deploy to staging environment
2. ✅ Performance optimization
3. ✅ Hardware integration planning
4. ✅ User acceptance testing
5. ✅ Production release

---

**Testing Status**: Ready for comprehensive desktop testing  
**Hardware Status**: Awaiting AR glasses hardware  
**Backend Status**: Optional (UI works in mock mode)
