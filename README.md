# AksharFlow (अक्षर-फ्लो)

Free, open-source, offline-first typing studio for **English (QWERTY)** and **Hindi (InScript & Remington GAIL)** built with **Tauri v2**, a **Rust (2024 Edition)** core workspace, and **React 19 + TypeScript + Tailwind CSS v4**.

## Key Features

- **Jungle Journey (700 Structured Lessons per Layout)**: Deterministic 28-unit curriculum (`KeyIntro`, `Drill`, `Words`, `Sentences`, `ReviewChallenge`, and `MiniGame` every 10th lesson) progressing from Home Row to high-speed stories with 1-to-5 star progression (`>= 3` stars unlocks the next lesson).
- **Canvas Balloon Pop Mini-Game**: Falling-word waves routed through the exact Rust layout/IME keystroke pipeline (`resolve_game_keystroke`), so English QWERTY, Hindi InScript, and Remington GAIL `ि` composition all work natively inside the game.
- **Zero OS Layout Dependency**: Intercepts physical `KeyboardEvent.code` events internally so Hindi InScript and CPCT/SSC Remington GAIL work regardless of your operating system's active keyboard layout, with `CapsLock` ignored on Hindi layers.
- **Authentic Remington GAIL IME & Devanagari Graphemes**: Supports visual-order `ि` reordering across consonant clusters (`f + D + r` to `क्ति`), half-consonant virama stripping (`+ k`), Reph (`Shift+Z`), `Space` IME flush, and `unicode-segmentation` cluster highlighting.
- **Smart Adaptive Engine (Keybr-Style)**: Tracks per-key error rate and EMA latency (capped at 2500 ms), progressively unlocks keys starting from Home Row, and dynamically weights lines 60% toward your top 3 weakest keys.
- **CPCT / SSC Timed Exam Mode**: 1, 2, 5, 10, and 15-minute timed tests with bundled Hindi/English passages, custom saved passages, strict no-Backspace mode, and Net/Gross WPM + KPH scorecards.
- **Analytics & Local Portability**: Chart.js WPM & Accuracy trend graphs, Jungle Star progress summary, Top 5 Weakest Keys drill launcher, Red-to-Green keycap heatmap, SVG 10-finger posture guide, mechanical switch Web Audio synth, bilingual `EN | हिं` UI, Zen Focus mode, and `.json` backup export/import backed by WAL SQLite (`v1` backup compatible).

## Architecture

- `crates/aksharflow-core`: Pure UI-agnostic Rust crate (`keymap`, `grapheme`, `curriculum`, `adaptive`, `corpus`, `metrics`, `session`, `storage`).
- `src-tauri`: Tauri v2 desktop host exposing typed IPC commands via `tauri-specta`.
- `src/`: React 19 + Vite + Tailwind CSS v4 + Zustand UI with bundled offline fonts (`Noto Sans Devanagari`, `JetBrains Mono`, `Inter`).

## Prerequisites

- **Rust** (`1.90+`, Edition 2024) & Cargo
- **Node.js** (`20+`) & **pnpm** (`10+`)
- **Tauri v2 Linux system libraries** (on Linux: `webkit2gtk-4.1`, `gtk+-3.0`)

## Development & Verification

```bash
# Install frontend dependencies
pnpm install

# Launch desktop app in dev mode (hot-reloads Vite + Rust backend)
pnpm tauri dev

# Run Rust core unit & integration tests + export Specta TypeScript bindings
cargo test --workspace

# Run frontend unit tests
pnpm test

# Typecheck & bundle frontend
pnpm build

# Build Tauri desktop application
pnpm tauri build
```

## Shortcuts

- `Tab` then `Enter`: Load a new practice line or restart the current lesson
- `Ctrl + Backspace`: Clear the preceding grapheme cluster
- `Esc` (Mini-Game): Unlock the currently targeted falling balloon word

## License

Licensed under the **GNU Affero General Public License v3.0 (AGPLv3)**. See [LICENSE](LICENSE).
