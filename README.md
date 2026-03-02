# insomniAPP

A lightweight desktop application that keeps your computer awake by simulating activity when you're idle. Built with Tauri 2, React, and Rust for native performance with a minimal footprint.

---

## What It Does

insomniAPP prevents your computer from locking the screen, going to sleep, or showing you as "Away" in chat applications. It monitors your system's idle time and, when you've been inactive long enough, silently presses the **F15 key** at regular intervals to keep the system awake — a key that is virtually never mapped to anything, so it won't interfere with your work.

The app is smart enough to **automatically pause when you're in a meeting** (microphone or camera active), so it won't fight with video calls or screen shares.

---

## Features

### Keep-Awake Engine
- Monitors system idle time every 3 seconds using native OS APIs
- Simulates the F15 key press at configurable intervals to prevent sleep/lock
- Distinguishes between real user inactivity and its own simulated input using a grace-period algorithm
- Tracks actual idle duration accurately, even while simulating activity

### Meeting Detection
- Automatically detects active microphone and camera usage
- Pauses all activity simulation during meetings — no interference with calls
- **Windows**: Reads the Windows Registry (`CapabilityAccessManager`) to check mic/webcam consent status
- **macOS**: Uses `pgrep` for camera processes (`VDCAssistant`, `AppleCameraAssistant`) and `ioreg` for audio input state

### System Tray Integration
- Lives in the system tray for an unobtrusive experience
- Left-click the tray icon to show the compact status window
- Right-click context menu with Toggle, Show Window, and Quit options
- Closing the window hides it to the tray instead of quitting the app

### Compact Status Window
- Ultra-compact UI (240x78 pixels) — stays out of your way
- Positions itself at the bottom-left corner of your screen
- Non-resizable for a consistent, minimal footprint

### Real-Time Status Indicators
Four distinct states with color-coded indicators:

| Status | Color | Meaning |
|---|---|---|
| **Disabled** | Gray | App is toggled off |
| **Paused** | Yellow | In a meeting (mic/camera active) |
| **Monitoring** | Blue | Watching idle time, not yet simulating |
| **Active** | Green (pulsing) | Currently simulating activity |

### Idle Timer Display
- Shows current idle time in `MM:SS` format
- Visual progress bar indicating how close you are to the idle threshold

### Configurable Settings
| Setting | Range | Default | Description |
|---|---|---|---|
| **Idle Threshold** | 10–600 seconds | 30s | How long you must be inactive before simulation starts |
| **Simulation Interval** | 5–300 seconds | 15s | How often F15 is pressed while simulating |

Settings are applied in real-time with a 500ms debounce — no restart needed.

### Theme Support
- Dark and Light mode toggle
- Preference persists during the session

---

## How It Works

### Architecture Overview

```
┌─────────────────────────────────────┐
│           React Frontend            │
│  (StatusPanel, IdleTimer, Settings) │
│         Tailwind CSS + Vite         │
└──────────────┬──────────────────────┘
               │  Tauri Events & Commands
┌──────────────┴──────────────────────┐
│           Rust Backend              │
│  ┌───────────┐  ┌────────────────┐  │
│  │KeepAwake  │  │  Tray Manager  │  │
│  │  Engine   │  │                │  │
│  └─────┬─────┘  └────────────────┘  │
│        │                            │
│  ┌─────┴─────────────────────────┐  │
│  │   Platform Layer (Windows/Mac)│  │
│  │  - Idle Detection             │  │
│  │  - Meeting Detection          │  │
│  │  - Input Simulation (F15)     │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

### Engine Loop (every 3 seconds)

1. **Read OS idle time** — calls platform-specific APIs to get seconds since last real user input
2. **Detect genuine activity** — if OS idle is under 5 seconds AND at least 5 seconds have passed since the last F15 simulation, the user is genuinely active (this grace period prevents the app from being fooled by its own key presses)
3. **Calculate effective idle time** — uses tracked idle time once the threshold is crossed, ensuring accurate duration even while simulating
4. **Check meeting status** — queries mic and camera state through native APIs
5. **Decide whether to simulate** — only simulates when: app is enabled AND user is idle AND not in a meeting AND enough time has passed since the last simulation
6. **Simulate F15** — presses the F15 key if all conditions are met
7. **Emit status update** — sends the current state to the frontend via Tauri events

### Platform-Specific Implementations

**Windows**
- **Idle Detection**: `GetLastInputInfo` + `GetTickCount` Win32 APIs
- **Meeting Detection**: Windows Registry `CapabilityAccessManager\ConsentStore` — checks `LastUsedTimeStop` values for microphone and webcam across both packaged (UWP) and non-packaged (desktop) applications
- **Input Simulation**: `enigo` crate with F15 key

**macOS**
- **Idle Detection**: `CGEventSourceSecondsSinceLastEventType` from CoreGraphics
- **Meeting Detection**: `pgrep` for `VDCAssistant` / `AppleCameraAssistant` (camera) and `ioreg` for `AppleHDAEngineInput` (microphone)
- **Input Simulation**: `enigo` crate with F15 key

---

## Tech Stack

### Frontend
| Technology | Version | Purpose |
|---|---|---|
| **React** | 19 | UI framework |
| **TypeScript** | 5.9 | Type-safe JavaScript |
| **Tailwind CSS** | 4 | Utility-first styling |
| **Vite** | 7 | Build tool and dev server |
| **@tauri-apps/api** | 2 | Frontend-to-backend communication |

### Backend
| Technology | Version | Purpose |
|---|---|---|
| **Tauri** | 2 | Desktop application framework |
| **Rust** | 2021 Edition | Systems language for performance |
| **enigo** | 0.6 | Cross-platform input simulation |
| **tokio** | 1 | Async runtime for the engine loop |
| **serde** | 1 | Serialization between frontend and backend |
| **windows** | 0.62 | Windows API bindings |
| **winreg** | 0.55 | Windows Registry access |
| **core-foundation** | 0.10 | macOS native APIs |
| **coreaudio-sys** | 0.2 | macOS audio system APIs |

---

## Project Structure

```
insomniapp/
├── src/                              # React Frontend
│   ├── main.tsx                      # App entry point
│   ├── App.tsx                       # Root component
│   ├── types.ts                      # TypeScript interfaces (AppStatus)
│   ├── styles.css                    # Tailwind CSS imports
│   ├── components/
│   │   ├── StatusPanel.tsx           # Logo, status indicator, theme toggle, enable/disable button
│   │   ├── IdleTimer.tsx             # Idle time display with progress bar
│   │   ├── MeetingIndicator.tsx      # Meeting status (mic/camera active)
│   │   └── SettingsForm.tsx          # Idle threshold and interval inputs
│   ├── contexts/
│   │   └── ThemeContext.tsx           # Dark/Light theme state
│   └── hooks/
│       └── useAppState.ts            # Status polling and event listener
│
├── src-tauri/                        # Rust Backend
│   ├── src/
│   │   ├── main.rs                   # Application entry point
│   │   ├── lib.rs                    # Tauri app setup, event handlers, tray initialization
│   │   ├── commands.rs               # Tauri IPC commands (get_status, toggle, update_settings)
│   │   ├── state.rs                  # AppStatus struct and thread-safe AppState
│   │   ├── keepawake.rs              # Core engine loop (idle detection + F15 simulation)
│   │   ├── idle.rs                   # Platform-agnostic idle detection wrapper
│   │   ├── meeting.rs                # Platform-agnostic meeting detection wrapper
│   │   ├── tray.rs                   # System tray setup and event handling
│   │   └── platform/
│   │       ├── mod.rs                # Platform module router
│   │       ├── windows.rs            # Windows: GetLastInputInfo, Registry-based mic/camera
│   │       └── macos.rs              # macOS: CGEventSource, pgrep/ioreg-based detection
│   ├── Cargo.toml                    # Rust dependencies
│   ├── tauri.conf.json               # Tauri app configuration
│   └── icons/                        # App icons (PNG, ICO, ICNS)
│
├── package.json                      # Node.js dependencies and scripts
├── vite.config.ts                    # Vite build configuration
├── tsconfig.json                     # TypeScript configuration
├── index.html                        # HTML template
└── README.md                         # This file
```

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Tauri CLI prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/insomniapp.git
cd insomniapp

# Install frontend dependencies
npm install
```

### Development

```bash
# Start in development mode (launches both Vite dev server and Tauri window)
npm run tauri dev
```

### Build for Production

```bash
# Build the production application
npm run tauri build
```

The built application will be in `src-tauri/target/release/bundle/`.

---

## IPC Commands

The frontend communicates with the Rust backend through three Tauri commands:

| Command | Parameters | Returns | Description |
|---|---|---|---|
| `get_status` | — | `AppStatus` | Returns the current application state |
| `toggle_enabled` | — | `AppStatus` | Toggles the enabled flag and returns updated state |
| `update_settings` | `idle_threshold_secs?: number`, `simulation_interval_secs?: number` | `AppStatus` | Updates one or both settings and returns updated state |

### AppStatus Object

```typescript
interface AppStatus {
  enabled: boolean;              // Whether the app is active
  isIdle: boolean;               // Whether the user is considered idle
  idleSeconds: number;           // Current idle time in seconds
  isInMeeting: boolean;          // Whether a meeting is detected (mic/camera)
  isSimulating: boolean;         // Whether the app is actively simulating input
  idleThresholdSecs: number;     // Configured idle threshold
  simulationIntervalSecs: number; // Configured simulation interval
}
```

### Events

| Event | Payload | Direction | Description |
|---|---|---|---|
| `status-update` | `AppStatus` | Backend → Frontend | Emitted every 3 seconds with the latest state |

---

## Why F15?

The F15 key was chosen because:
- It exists on virtually all keyboard layouts at the OS level
- It is almost never mapped to any application action
- It resets the OS idle timer just like any other key press
- It won't type characters, trigger shortcuts, or interfere with your work
- It's the least intrusive way to keep a system awake

---

## License

MIT
