# Backends

Source of truth: `src/backend/mod.rs` doc comment (verbatim tables below)
and the per-backend sources. All three implement Ratatui's `Backend` plus
Ratzilla's `WebEventHandler`, and the `CellSized` trait (physical vs CSS
pixel cell dimensions, `src/backend/cell_sized.rs`).

## Feature comparison (verbatim from src/backend/mod.rs:21-34)

| Feature                      | DomBackend | CanvasBackend | WebGl2Backend  |
|------------------------------|------------|---------------|----------------|
| **60fps on large terminals** | ✗          | ✗             | ✓              |
| **Memory Usage**             | Highest    | Medium        | Lowest         |
| **Hyperlinks**               | ✗          | ✗             | ✓              |
| **Text Selection**           | Linear     | ✗             | Linear/Block   |
| **Unicode/Emoji Support**    | Full       | Limited²      | Full¹          |
| **Dynamic Characters**       | ✓          | ✓             | ✓¹             |
| **Font Variants**            | ✓          | Regular only  | ✓              |
| **Underline**                | ✓          | ✗             | ✓              |
| **Strikethrough**            | ✓          | ✗             | ✓              |
| **Browser Support**          | All        | All           | Modern (2017+) |
| **Mouse Events**             | Full       | Full          | Basic          |

¹ Dynamic font atlas rasterizes glyphs on demand (full Unicode, emoji, and
font-variant support); the static atlas is limited to glyphs compiled into
the `.atlas` file. ² Unicode is supported, but emoji only render correctly
when they span one cell — most emoji occupy two cells.

Mouse-event support per backend (verbatim from src/backend/mod.rs:46-54):

| Event Type    | DomBackend | CanvasBackend | WebGl2Backend |
|---------------|------------|---------------|---------------|
| `Moved`       | ✓          | ✓             | ✓             |
| `ButtonDown`  | ✓          | ✓             | ✓             |
| `ButtonUp`    | ✓          | ✓             | ✓             |
| `SingleClick` | ✓          | ✓             | ✗             |
| `DoubleClick` | ✓          | ✓             | ✗             |
| `Entered`     | ✓          | ✓             | ✗             |
| `Exited`      | ✓          | ✓             | ✗             |

> Unverified: `webgl2.rs`'s own doc and its `From<&TerminalMouseEvent>`
> conversion (src/backend/webgl2.rs:949-973) DO map SingleClick/Entered/
> Exited for the WebGL2 backend, contradicting the table above. Authoritative
> behavior for WebGl2's click/enter/leave events is unclear — don't rely on
> them without a runtime check.

## How each one renders

- **DomBackend** turns each buffer `Cell` into a `<span>` inside per-row
  `<pre>` elements, appended to a grid `<div id="grid">` (`src/backend/dom.rs`).
  It rebuilds the grid on window resize. Cell size is *measured* with a probe
  `<span>` + `getBoundingClientRect()` (falls back to 10×20px,
  `src/backend/dom.rs:186-209`). As real HTML it inherits the page's CSS —
  font-family on `pre` in your `index.html` sizes the grid.
- **CanvasBackend** draws to a 2D canvas via `CanvasRenderingContext2d`
  (`src/backend/canvas.rs`), diff-copying buffer changes (`BitVec` dirty
  tracking). Cell size is **hardcoded** `CELL_WIDTH = 10.0`,
  `CELL_HEIGHT = 19.0` with a 5px translate offset (`src/backend/canvas.rs:35-41,245`)
  — a non-monospace-16ish font misaligns the grid. `always_clip_cells` clips
  each glyph to its cell (helps out-of-bounds fonts, costs perf). A private
  `RowColorOptimizer` batches same-color adjacent cells into fill rects.
- **WebGl2Backend** wraps `beamterm_renderer::Terminal`, uploads changed
  cells to GPU buffers, calls `render_frame()` (`src/backend/webgl2.rs`).
  Glyphs from a font atlas: **static** (pre-generated `.atlas`, default,
  missing chars render as fallback glyph) or **dynamic** (rasterized on
  demand — full Unicode/emoji). Perf marks `sync-terminal-buffer` /
  `webgl-render` when `measure_performance(true)`.

## Options builders

**Import paths matter.** `CanvasBackend`/`DomBackend`/`WebGl2Backend` are
re-exported at the crate root, but their `*Options` builders are **not** —
they live in the backend submodules only (`src/lib.rs` re-exports the structs,
none of the options types; import them as `examples/shared/src/backend.rs:3`):

```rust
use ratzilla::backend::{
    canvas::CanvasBackendOptions,
    dom::DomBackendOptions,
    webgl2::WebGl2BackendOptions,
};
```

All three backends have a builder; `new()` and `new_with_size()` are
convenience wrappers over `new_with_options(...)`.

- **`DomBackendOptions::new(grid_id: Option<String>, cursor_shape)`**
  (`src/backend/dom.rs:36-68`). Grid id defaults to `"grid"` or the given id
  suffixed with `_ratzilla_grid`. See also `DomBackend::new_by_id(id)`.
- **`CanvasBackendOptions::new()`** + `.grid_id(id)` + `.size((w, h))`
  (`src/backend/canvas.rs:44-74`). Size overrides auto-detection. Post-construction:
  `set_background_color(Color)`, `set_cursor_shape`, and
  `set_debug_mode(Some("#666") | Some("red") | None)` for layout debugging.
- **`WebGl2BackendOptions::new()`** is the biggest (`src/backend/webgl2.rs:98-252`):
  - `.grid_id(id)`, `.size((w,h))`, `.fallback_glyph("?")`
  - `.font_atlas_config(FontAtlasConfig::dynamic(&["Maple Mono NF CN"], 15.0))`
    for a dynamic atlas, or `FontAtlasConfig::Static(atlas)` for a
    pre-generated one. `font_atlas()` (static only) is **deprecated since 0.3.0**.
  - `.cursor_shape(CursorShape)` — shapes at
    `ratzilla::backend::cursor::CursorShape` / root `CursorShape`:
    `SteadyBlock` (default, flips colors), `SteadyUnderScore` (toggles
    underline), `None`.
  - `.enable_mouse_selection_with_mode(SelectionMode::Block | SelectionMode::Linear)`
    — mouse text selection with copy-on-select; `enable_mouse_selection()` is
    **deprecated since 0.3.0**.
  - `.enable_hyperlinks()` — click opens in `_blank`; override with
    `.on_hyperlink_click(|url| ...)`.
  - `.canvas_padding_color(Color)`, `.measure_performance(bool)`,
    `.enable_console_debug_api()` (`window.__beamterm_debug`),
    `.disable_auto_css_resize()` (external CSS controls canvas size).
  - Instance methods: `resize_canvas()`, `set_size(w,h)`, `options()`;
    `cell_size()` **deprecated** (says since 0.4.0) — use
    `CellSized::cell_size_px()`.

Prefer `FontAtlasConfig::dynamic` when you know the on-page font; the
static default atlas renders missing glyphs as a space.

## Sharing cell metrics: `CellSized`

`cell_size_px()` returns physical (device) pixels, useful for GPU work;
`cell_size_css_px()` returns logical pixels, useful for DOM positioning and
mouse-coordinate math (`src/backend/cell_sized.rs`). Consult both for
pixel-precise work on HiDPI (DPR > 1) displays.

## The `MultiBackendBuilder` pattern (from examples, undocumented in README)

`examples/shared/src/backend.rs` ships an example-only helper the real
examples all use, so users can switch backends without editing code:

- `BackendType { Dom, Canvas, WebGl2 }` (default Dom).
- `RatzillaBackend` — enum wrapping all three backends, implementing
  `Backend` + `WebEventHandler` for runtime switching.
- `FpsTrackingBackend` — wraps `RatzillaBackend`, records frame timing per
  `flush()` (thread_local ring buffer, `examples/shared/src/fps.rs`).
- `MultiBackendBuilder::with_fallback(BackendType)`
  → `.terminal_options(TerminalOptions)`
  → `.canvas_options(...)` / `.dom_options(...)` / `.webgl2_options(...)`
  → `.build_terminal() -> io::Result<Terminal<FpsTrackingBackend>>`. Also
  injects a floating "Backend: … — FPS:" footer bar.

```rust
// examples/minimal/src/main.rs (shape): switch backends w/o editing code,
// ?backend=dom|canvas|webgl2 URL param → .with_fallback(...) → Dom
let mut terminal = MultiBackendBuilder::with_fallback(BackendType::Dom)
    .webgl2_options(WebGl2BackendOptions::new()
        .enable_console_debug_api()
        .enable_mouse_selection_with_mode(SelectionMode::Block))
    .build_terminal()?;
```

Copy this for switchable backends; for a single fixed backend,
`Terminal::new(DomBackend::new()?)` from the README is simpler.

## Gotchas

- `CanvasBackend::window_size()` is `unimplemented!()` — panics if called
  (`src/backend/canvas.rs:543-545`).
- WebGL2 needs a modern browser (Chrome 56+/Firefox 51+/Safari 15+); no
  built-in fallback — graceful degradation to Canvas is on you.
- WebGL2 keys: canvas gets `tabindex="0"`; it must have focus for keys to
  arrive.