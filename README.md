# insomniAPP

A lightweight desktop application that keeps your computer awake by simulating activity when you're idle. Built with Tauri 2, React, and Rust for native performance with a minimal footprint.

---

## What It Does

insomniAPP prevents your computer from locking the screen, going to sleep, or showing you as "Away" in chat applications. It monitors your system's idle time and, when you've been inactive long enough, silently presses the **F15 key** at regular intervals to keep the system awake — a key that is virtually never mapped to anything, so it won't interfere with your work.

---

## Features

### Keep-Awake Engine
- Monitors system idle time every 3 seconds using native OS APIs
- Simulates the F15 key press at configurable intervals to prevent sleep/lock
- Distinguishes between real user inactivity and its own simulated input using a grace-period algorithm
- Tracks actual idle duration accurately, even while simulating activity

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
| **Paused** | Yellow | Screen locked or display turned off |
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
│  │  - Input Simulation (F15)     │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

### Engine Loop (every 3 seconds)

1. **Read OS idle time** — calls platform-specific APIs to get seconds since last real user input
2. **Detect genuine activity** — if OS idle is under 5 seconds AND at least 5 seconds have passed since the last F15 simulation, the user is genuinely active (this grace period prevents the app from being fooled by its own key presses)
3. **Calculate effective idle time** — uses tracked idle time once the threshold is crossed, ensuring accurate duration even while simulating
4. **Decide whether to simulate** — only simulates when: app is enabled AND user is idle AND enough time has passed since the last simulation
5. **Simulate F15** — presses the F15 key if all conditions are met
6. **Emit status update** — sends the current state to the frontend via Tauri events

### Platform-Specific Implementations

**Windows**
- **Idle Detection**: `GetLastInputInfo` + `GetTickCount` Win32 APIs
- **Session/Display State**: Win32 desktop and power-notification APIs to pause while the screen is locked or the display is off
- **Input Simulation**: `enigo` crate with F15 key

**macOS**
- **Idle Detection**: `CGEventSourceSecondsSinceLastEventType` from CoreGraphics
- **Input Simulation**: `enigo` crate with F15 key

---

## Tech Stack

### Frontend
| Technology | Version | Purpose |
|---|---|---|
| **React** | 19 | UI framework |
| **TypeScript** | 6 | Type-safe JavaScript |
| **Tailwind CSS** | 4 | Utility-first styling |
| **Vite** | 8 | Build tool and dev server |
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
| **core-foundation** | 0.10 | macOS native APIs |

---

## Project Structure

Each Rust module is split into the logic it owns and the adapter that talks to
the operating system or to Tauri. The logic is pure and unit-tested; the adapter
is a thin wrapper that cannot run without a real windowing system.

```
insomniapp/
├── frontend/
│   ├── web/                              # Marketing site (React + Vite)
│   │   ├── src/                          # Components, sections, and their *.test.tsx
│   │   └── vitest.config.ts              # jsdom + V8 coverage, 100% thresholds
│   │
│   └── desktop/                          # Desktop app
│       ├── src/                          # React frontend and its *.test.tsx
│       │   ├── main.tsx                  # App entry point
│       │   ├── App.tsx                   # Root component
│       │   ├── types.ts                  # TypeScript interfaces (AppStatus)
│       │   ├── components/               # StatusPanel, IdleTimer, SettingsForm, DisguiseWindow
│       │   ├── contexts/                 # ThemeContext
│       │   └── hooks/                    # useAppState, useDisguiseState, useUpdateCheck
│       ├── vitest.config.ts              # jsdom + V8 coverage, 100% thresholds
│       │
│       └── src-tauri/                    # Rust backend
│           ├── .cargo/config.toml        # `cargo coverage` alias (exclusion list)
│           ├── src/
│           │   ├── main.rs               # Process entry point                 [adapter]
│           │   ├── lib.rs                # Tauri builder wiring                [adapter]
│           │   ├── commands.rs           # Tauri IPC command wrappers          [adapter]
│           │   ├── state.rs              # AppStatus, AppState, settings rules
│           │   ├── keepawake/
│           │   │   ├── engine.rs         # Pure tick decision procedure
│           │   │   └── mod.rs            # Async loop, F15 simulation          [adapter]
│           │   ├── tray/
│           │   │   ├── layout.rs         # Pure window placement
│           │   │   └── mod.rs            # Tray icon and menu                  [adapter]
│           │   ├── disguise/
│           │   │   ├── name.rs           # Sanitising, persistence format, AppUserModelID
│           │   │   ├── store.rs          # Reading/writing the state file
│           │   │   ├── process_name.rs   # Windows process-name helpers
│           │   │   ├── enumerate.rs      # Win32 EnumWindows                   [adapter]
│           │   │   └── mod.rs            # AppHandle glue                      [adapter]
│           │   └── platform/
│           │       ├── mod.rs            # Per-OS router
│           │       ├── convert.rs        # Pure idle/display-state conversions
│           │       ├── fallback.rs       # Shims for other targets
│           │       ├── windows.rs        # GetLastInputInfo, desktop, power    [adapter]
│           │       └── macos.rs          # CGEventSource idle detection        [adapter]
│           ├── Cargo.toml                # Rust dependencies
│           ├── tauri.conf.json           # Tauri app configuration
│           └── icons/                    # App icons (PNG, ICO, ICNS)
│
├── .github/workflows/
│   ├── ci.yml                            # Tests and coverage gates
│   └── release.yml                       # Signed installers
└── README.md                             # This file
```

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (^20.19.0 or >=22.12.0)
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

## Testing and Coverage

Every test command enforces a **100% coverage gate** and fails the build below it.
The same gates run on every pull request in `.github/workflows/ci.yml`.

### Frontend (Vitest + React Testing Library)

Both React packages use Vitest with the V8 coverage provider and a `jsdom`
environment, with thresholds of 100% for lines, statements, functions, and
branches.

```bash
# Marketing site
cd frontend/web
npm ci
npm run test            # watch mode
npm run test:coverage   # single run, enforces the 100% gate

# Desktop frontend (Tauri APIs are mocked in src/test/setup.ts)
cd frontend/desktop
npm ci
npm run test:coverage
```

### Rust backend (cargo-llvm-cov)

```bash
cargo install cargo-llvm-cov --locked   # one-off
rustup component add llvm-tools-preview # one-off

cd frontend/desktop/src-tauri
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo coverage --summary-only --fail-under-lines 100
```

`cargo coverage` is an alias defined in `src-tauri/.cargo/config.toml`. It pins
the exclusion list below so contributors and CI measure exactly the same files.
Running `cargo llvm-cov` directly, without that alias, measures the adapters too
and will not reach 100%.

> The Rust crate embeds the built frontend via `tauri::generate_context!`, so run
> `npm run build` in `frontend/desktop` at least once before `cargo test`.

### Inspecting HTML reports

```bash
# Frontend: writes coverage/index.html
cd frontend/web && npm run test:coverage && open coverage/index.html

# Rust: opens target/llvm-cov/html/index.html
cd frontend/desktop/src-tauri && cargo coverage --html --open
```

CI also uploads every report as a build artefact (`coverage-web`,
`coverage-desktop`, `coverage-rust-<os>`) so a failed gate can be diagnosed
without reproducing it locally.

### Running every gate locally

```bash
(cd frontend/web       && npm ci && npm run build && npm run test:coverage)
(cd frontend/desktop   && npm ci && npm run build && npm run test:coverage)
(cd frontend/desktop/src-tauri \
   && cargo fmt --check \
   && cargo clippy --all-targets --all-features -- -D warnings \
   && cargo coverage --summary-only --fail-under-lines 100)
```

### What counts as an acceptable exclusion

A file may be excluded from coverage **only** when every line in it is an
irreducible adapter: a call into the operating system, the Tauri runtime, or the
process bootstrap, with no decision of our own left in it. Before excluding
anything, the logic must first be extracted into a module that *is* covered.

Exclusion means "not subject to the 100% gate", not "untested". The OS adapters
still carry smoke tests that call them for real; they are excluded because no
single run can exercise both branches of, say, `is_session_locked`, whose answer
depends on whether the host session is interactive.

Platform-specific Rust code is covered on the platform that compiles it: the CI
matrix runs the Rust gate on Linux, Windows, and macOS.

Currently excluded, all pinned in `src-tauri/.cargo/config.toml`:

| File | Why it cannot reach 100% | Logic extracted to | Smoke-tested |
|---|---|---|---|
| `main.rs`, `lib.rs` | Process entry point and Tauri builder wiring | — | no |
| `commands.rs` | `#[tauri::command]` wrappers that only unlock state and delegate | `state.rs`, `disguise/` | payload contract |
| `keepawake/mod.rs` | Unbounded async loop, clock, and `enigo` input simulation | `keepawake/engine.rs` | no |
| `tray/mod.rs` | Tray icon, menu, and window APIs | `tray/layout.rs` | no |
| `disguise/mod.rs` | `AppHandle` access to the data directory, window, and tray | `disguise/name.rs`, `disguise/store.rs` | no |
| `disguise/enumerate.rs` | Win32 `EnumWindows`; result depends on which windows are open | `disguise/process_name.rs` | yes |
| `platform/windows.rs` | `unsafe` Win32 idle, desktop, and power-notification calls | `platform/convert.rs` | yes |
| `platform/macos.rs` | `unsafe` CoreGraphics FFI | `platform/convert.rs` | yes |

Tauri's `tauri::test` mock runtime is deliberately **not** used. Constructing any
mock `App` links `muda`, which imports `TaskDialogIndirect` from comctl32 v6.
Only the bundled app gets an activation-context manifest selecting v6, so on
Windows the `cargo test` harness aborts at load with `STATUS_ENTRYPOINT_NOT_FOUND`
before any test runs, and Cargo's `rustc-link-arg-tests` cannot reach the lib's
unit-test harness to fix it. `commands.rs` instead pins the one thing it owns
outright — the camelCase `SettingsPayload` wire format.

On the frontend, only test files, the test setup directory, and type-only
declarations (`vite-env.d.ts`, `types.ts`) are excluded.

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
  isSessionLocked: boolean;      // Whether the session is locked (pauses simulation)
  isDisplayOff: boolean;         // Whether the display is off (pauses simulation)
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
