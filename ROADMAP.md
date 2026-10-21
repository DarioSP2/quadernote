# QuaderNote Roadmap & Milestone Queue

## Current Focus: Milestone 0 — Core Workspace & Engine Foundation

### M0: Foundation & Deterministic Engine (Infrastructure First)
- [x] **M0.1 Workspace Initialization:** Initialize Cargo workspace with `crates/quader-geom`, `crates/quader-doc`, `crates/quader-engine`, and `apps/quadernote-cli`, enforcing clean `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`.
- [x] **M0.2 Spatial Geometry (`quader-geom`):** Implement `PointMm`, `SizeMm`, `RectMm`, `ViewportTransform` (pan/zoom in millimeters), and 2D spatial hit-testing (`RectMm::intersects`, `RectMm::contains`, and bounding-box queries) with 100% unit test coverage.
- [x] **M0.3 Document, Ink & Attachment Model (`quader-doc`):** Implement `Notebook`, `Section`, `Page` (with `indent_level` 0..=2), `PaperRuleStyle` (`None`, `RuledLines`, `Grid`, `A4Guide210Mm`), `Outline` containers, `CanvasAttachment`, `PrintoutStack`, `EmbeddedSheet`, `NoteTag`, and pressure-sensitive `InkStroke` structures + JSON round-trip serialization tests.
- [x] **M0.4 Command Engine & Undo/Redo (`quader-engine`):** Implement `EngineSession`, transactional snapshot undo/redo stack, full-text/tag search (`search_text`), and core commands for sections, pages, outlines, tags, ink, attachments, printouts, and sheets.
- [ ] **M0.5 Headless CLI & MCP Server (`quader-automation`, `quadernote-cli`):** Expose `inspect_notebook`, `execute_command`, `search_notes`, and batch command runner over stdio and TCP `127.0.0.1:7982`.
- [ ] **M0.6 Native Shell & Offscreen Test Harness (`quader-ui-egui`, `quadernote`):** Render the 4-region shell (Top Ribbon, Left Section Rail, Right Page Hierarchy, Center 2D Infinite Canvas) and an offscreen screenshot example (`shot.rs`) for agent verification.

### M1: Infinite Canvas, Rich Outlines & Smart Clipboard (Easy Wins)
- [ ] **M1.1 Free-Floating Outline Placement:** Click anywhere on the 2D canvas to place or focus an `Outline` box at `(x_mm, y_mm)`; drag the top handle to reposition.
- [ ] **M1.2 Rich Text & Formatting Commands:** Implement `format.bold`, `format.italic`, `format.underline`, `format.strikethrough`, `format.highlight`, bulleted/numbered lists, and font size in `mm`.
- [ ] **M1.3 Note Tags Catalog:** Implement interactive `Todo` checkboxes, `Important` stars, `Question` markers, and tag summary filtering (`search.by_tag`).
- [ ] **M1.4 Canvas Tables:** Insert 2D tables inside any `Outline` (`table.insert`, `table.add_row`, `table.add_col`, `table.shade_cell`), matching OneNote's `Tab`-to-create-column behavior.
- [ ] **M1.5 Smart Clipboard Paste & Global Preset Library (`clipboard.paste_smart`, `library.save_snippet`):** Paste images, text, or tables directly onto the canvas at the active cursor/pen coordinate (`Ctrl+V`) and save reusable diagram/pen presets across notebooks.

### M2: Stylus, Handwriting & Microsoft Journal Gestures (`Draw` Tab)
- [ ] **M2.1 Pressure-Sensitive Pen, Highlighter & Touch Routing:**
  - Stylus input draws smooth Catmull-Rom splines with pressure-varied stroke width in `mm`.
  - Single-finger touch pans the infinite canvas (`viewport.pan`) with inertial fling even when the Pen tool is active.
  - Two-finger pinch zooms the canvas (`viewport.zoom`).
- [ ] **M2.2 Surface Pen Hardware Eraser & Stroke/Point Eraser:** Map hardware tail-eraser events (`PointerButton::Eraser`) to `ink.erase_stroke` (touch any segment to remove whole stroke) and `ink.erase_point` (split stroke at eraser radius in `mm`).
- [ ] **M2.3 Lasso Selection (`ink.lasso_select`):** Freeform polygon selection of ink strokes, outlines, and attachments to move, resize, or recolor as a group.
- [ ] **M2.4 Shape Recognition, Rule Lines & A4 Guides:** Snap hand-drawn lines, rectangles, arrows, and coordinate axes; toggle `5 mm` / `8 mm` grid, ruled backgrounds, and `210 mm` A4 page-width guides.
- [ ] **M2.5 Microsoft Journal-Style Ink Gestures (`ink.gesture_scratch_erase`, `ink.dwell_shape_snap`, `ink.margin_star_tag`):**
  - Scratching back-and-forth over ink strokes automatically erases them without switching tools.
  - Holding the pen tip still (`400 ms` dwell) at the end of a stroke snaps it to a clean geometric shape.
  - Drawing a star in the left margin automatically attaches an `Important` tag.

### M3: File Attachments, Printouts & Embedded Spreadsheets (`Insert` Tab)
- [ ] **M3.1 Arbitrary File Attachments (`attachment.insert`, `attachment.open`, `attachment.save_as`, `attachment.remove`):** Store attached files in the notebook's local `attachments/` content-addressed store (SHA-256 hash) and render a draggable `45 mm` x `14 mm` badge at `(x_mm, y_mm)`.
- [ ] **M3.2 PDF Printout (`printout.insert_pdf`, `printout.set_lock`, `printout.extract_text`):** Stack rendered PDF pages vertically (`190 mm` width, `10 mm` gap) with background locking and LOD tile caching, indexing embedded text into `search_notes`.
- [ ] **M3.3 PowerPoint Printout (`printout.insert_pptx`):** Parse `.pptx` slides (converting EMUs to millimeters at `1 mm = 36'000 EMUs`) and stack locked `190 mm`-wide slide frames vertically on the canvas.
- [ ] **M3.4 Excel Spreadsheet Display (`sheet.insert_xlsx`, `sheet.select_tab`, `sheet.refresh`, `sheet.convert_to_table`):** Parse `.xlsx`, `.xls`, `.ods`, and `.csv` via `calamine` and render an inline spreadsheet grid on the canvas with worksheet tab switches.

### M4: Storage, `.one` Interop, Vector PDF Export & Android APK
- [ ] **M4.1 Native `.qnote` Bundle:** Fast local folder/ZIP storage with human-readable JSON + Markdown mirror, `attachments/` folder, and binary ink buffers.
- [ ] **M4.2 Microsoft OneNote `.one` & `.onetoc2` Import:** Clean-room integration with `onenote_parser` (`>= 1.1.1`) to import existing OneNote sections, pages, outlines, attachments, printouts, and ink strokes.
- [ ] **M4.3 A4-Aware Vector PDF Export:** Export pages or sections to selectable vector PDFs (`krilla`) preserving millimeter scale without slicing containers in half.
- [ ] **M4.4 Cross-Platform Packaging:** Build Windows `.msi`/portable `.zip`, Linux Flatpak/AppImage/`.deb`, and Android `.apk` (`cargo-apk`).
