# AksharFlow (अक्षर-फ्लो)

Free, open-source, offline-first adaptive typing tutor for **English (QWERTY)** and **Hindi (Inscript & Remington GAIL)** built with **Tauri v2**, a pure **Rust (2024 Edition)** core engine, and **React 19 + TypeScript + Tailwind CSS v4**.

## Architecture

- `crates/aksharflow-core`: UI-agnostic Rust domain crate handling physical key interception, Hindi IMEs (Inscript & Remington GAIL), Unicode grapheme segmentation (`unicode-segmentation`), adaptive weak-key engine, offline dictionaries, real-time metrics, and embedded SQLite persistence.
- `src-tauri`: Tauri v2 desktop/mobile host exposing typed IPC commands via `tauri-specta`.
- `src/`: React 19 + Tailwind CSS v4 + Zustand UI with smooth grapheme rendering, touch-capable Virtual Keyboard & Finger Guide, Web Audio mechanical switch synthesis, and bilingual (`EN | हिं`) support.

## Development

```bash
pnpm install
pnpm tauri dev
```

## License

Licensed under the **GNU Affero General Public License v3.0 (AGPLv3)**. See [LICENSE](LICENSE).
