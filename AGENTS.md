# QuaderNote — Instructions for AI Agents

QuaderNote is a clean-room, open-source, Rust-native digital notebook and spatial 2D canvas targeting Microsoft OneNote and Microsoft Journal workflows — and going further on speed, offline privacy, and agent control. It runs natively on Windows, Linux, Android, macOS, and BSD, and in the browser via WebAssembly.

## 1. Start Every Session Here
1. Read `ROADMAP.md` (current milestone and next uncompleted task ID).
2. Execute strictly one task ID (or one explicitly requested GitHub Issue) at a time using the 5-Step Loop below.
3. Do not stop to ask the human owner for routine technical decisions. Only pause for human approval on:
   - Adding a dependency with a license outside `MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, or `Unicode-3.0`
   - Destructive git operations (`reset --hard`, force-push, deleting branches)
   - Modifying or deleting code during a Codebase Audit (always produce the audit report first and wait for approval)

## 2. Never Crash & Zero-Spaghetti Policy
People trust QuaderNote with their lecture notes, research, and handwritten ink; a crash loses their work.
- **No panics in library crates (`L0`–`L5`):** `.unwrap()`, `.expect()`, `todo!()`, `unimplemented!()`, and unchecked slice indexing (`slice[i]`) are forbidden outside `#[cfg(test)]` modules. Return typed `Result<T, Error>` values and recover gracefully in the command guard.
- **No dead or test-only production functions:** When refactoring a feature, delete the old implementation completely. Never leave obsolete functions alive solely because an old unit test still calls them.
- **Zero per-frame allocations in `L6`:** Never sort collections, lock heavy mutexes, or re-upload unchanged CPU RGBA buffers to GPU textures inside the per-frame `egui` `update()` loop. Cache layout and raster tiles and invalidate only on state change.
- **Regression tests:** Every bug or crash fix must include a test that reproduces the failure before the fix and passes after.
- **Privacy in logs:** Never log note text, ink coordinates, or file contents in error or panic messages.

## 3. Clean-Room Mandate
- Never inspect, disassemble, decompile, or copy anything from Microsoft OneNote or Microsoft Journal binaries, installers, shaders, or internal assets.
- Learn behavior exclusively from public documentation, published keyboard shortcut tables, user-observable behavior, and Microsoft's open file format specifications (`[MS-ONE]` and `[MS-ONESTORE]`).
- Never copy code from GPL/LGPL/AGPL projects. Every bundled font, icon, or sample asset must be permissive/open-licensed and recorded in `ATTRIBUTION.md` in the same commit.

## 4. Architecture & Layering
All spatial measurements in QuaderNote use the International System of Units (`mm` for canvas coordinates, stroke widths, and page dimensions).
- **L0 (`quader-geom`):** 2D points (`PointMm`), bounding boxes (`RectMm`), viewport pan/zoom transforms (`ViewportTransform`), and 2D spatial indexing in millimeters. Zero external UI dependencies.
- **L1 (`quader-doc`, `quader-ink`, `quader-fonts`, `quader-proof`):** Hierarchical document model (`Notebook` -> `Section` -> `Page`), 2D free-floating `Outline` containers, `CanvasAttachment`, `PrintoutStack`, `EmbeddedSheet`, pressure/tilt `InkStroke` splines, note tags (`Todo`, `Important`, `Question`), font shaping, and offline spellcheck.
- **L2 (`quader-layout`, `quader-one`, `quader-formats`, `quader-cos`):** 2D outline text wrapping, tables, ink bounding/simplification, `.one` / `.onetoc2` import, `.xlsx` / `.pptx` / PDF syntax parsing, and `.qnote` (JSON + Markdown mirror) serialization.
- **L3 (`quader-render`, `quader-pdf`, `quader-ocr`):** Offscreen 2D tile, PDF printout, & ink rasterizer, OCR, and selectable A4-aware vector PDF export.
- **L4 (`quader-engine`):** Session state, transactional undo/redo history, full-text + tag + printout search index, feature parity catalog (`catalog.rs`), and the central Command Registry (`commands.rs`).
- **L5 (`quader-automation`):** Headless CLI runner, TCP JSON control channel (`127.0.0.1:7982`), and Model Context Protocol (MCP) server.
- **L6 (`quader-ui-egui`):** Native GPU interface (`egui` + `wgpu`). **Rule:** Nothing below `L6` may depend on `egui`, `eframe`, `winit`, or `rfd`.

## 5. Every Feature Is a Command
- Every user action must be registered as a command in `crates/quader-engine` with a stable string `id` (e.g., `page.create`, `canvas.insert_outline`, `ink.add_stroke`, `printout.insert_pdf`, `sheet.insert_xlsx`).
- The GUI ribbon, keyboard shortcuts, command search bar, CLI (`quadernote-cli`), JSON control port, and MCP server all dispatch through the exact same command ID.
- Programmatic command calls must never block on OS modal file dialogs.

## 6. The 5-Step Agent Execution Loop
For every task in `ROADMAP.md`:
1. **Orient:** Read the task specification and inspect only the crates owned by that layer.
2. **Write Tests First:** Write the unit and integration tests—including an undo/redo round-trip test (`execute` -> `undo` -> assert state equals initial -> `redo` -> assert state equals post-execute)—before writing implementation code.
3. **Implement:** Write the minimum clean Rust code in the target crate and wire the command into `quader-engine` and `quader-ui-egui`.
4. **Verify:**
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
5. **Record:** Check off the completed task `[x]` in `ROADMAP.md` and commit with the task ID prefix (e.g., `M0.1: initialize Cargo workspace`).
