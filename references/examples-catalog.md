# Examples catalog

The repo's `examples/` workspace is a working-pattern catalog. Find the
example closest to what you're building and copy its skeleton. Sixteen
workspace members (`examples/Cargo.toml`); `tauri`'s `src-tauri/` is
excluded from the workspace.

Note: nearly every example builds its terminal through
`examples-shared`'s `MultiBackendBuilder` (undocumented in the README — see
`references/backends.md`), so the backend it *demonstrates* is the
"fallback" argument, overridable with `?backend=` in the URL.

| Example | What it demonstrates | Notable APIs / non-obvious bits |
|---|---|---|
| `animations` | tachyonfx sweep-in/coalesce text animation, Canvas backend | `EffectRenderer::render_effect`, `fx::sweep_in`/`fx::coalesce`, `EffectTimer`, `Clear` widget |
| `canvas_stress_test` | WebGL2 perf stress: 4 text-coloring strategies, FPS monitor | `.measure_performance(true)`, pre-generating `Paragraph<'static>` widgets to cut JS-heap GC |
| `canvas_waves` | Custom `WaveInterference` effect animation | `IntoEffect` on a hand-written effect struct, per-frame delta from `web_time::Instant`, `.grid_id("container")` |
| `clipboard` | Browser clipboard read/write (`Ctrl+C`/`Ctrl+V`) | `SelectionMode::Linear`, `wasm_bindgen_futures::spawn_local` inside `on_key_event`, `navigator.clipboard` |
| `colors_rgb` | Animated OKHSV→RGB color wheel rendered with half-blocks | custom `Widget for &mut ColorsWidget`; buf[][] `set_char('▀').set_fg().set_bg()` |
| `demo` | Port of Ratatui's official demo (app/ui/effects modules) | WebGL2 full setup: `.measure_performance(true)`, `.enable_console_debug_api()`, `.enable_mouse_selection()`, `.disable_auto_css_resize()` |
| `demo2` | Full tabbed "demo2" run (tabs/colors/destroy/theme modules) | `TerminalOptions { viewport: Viewport::Fixed(Rect::new(0,0,81,18)) }`, OKLab colors, `.enable_mouse_selection_with_mode(SelectionMode::default())` |
| `minimal` | Key + mouse event handling, hover-cell highlight | `f.buffer_mut()[(col,row)].set_bg(...)`, `MouseEventKind` handling, `SelectionMode::Block` |
| `pong` | Bouncing `canvas::Circle`, CursorShape, page title | `utils::set_document_title("RATATUI")`, `widgets::Hyperlink`, per-backend `.grid_id("container")` on all three backends |
| `shared` | **Library, not an app** — helpers used by the other examples | `MultiBackendBuilder`, `BackendType`, `RatzillaBackend` (runtime-switchable enum), `FpsTrackingBackend`, thread_local `FpsRecorder`, DOM backend-switcher footer |
| `tauri` | Same effect as `animations`, but shipped in a Tauri 2 desktop shell | `src-tauri/` (tauri 2.8.5, `tauri-plugin-log`, capabilities), build flow `trunk build` → `cargo tauri dev` |
| `text_area` | Full `tui-textarea` editor integration | exhaustive `KeyCode`→`tui_textarea::Key` conversion; requires the `[patch.crates-io]` git-rev pin in `examples/Cargo.toml:38-40` |
| `unicode` | CJK + emoji rendering | **dynamic font atlas** `FontAtlasConfig::dynamic(&["Maple Mono NF CN"], 15.0)` |
| `user_input` | Hand-rolled input state machine (`Normal`/`Editing`) | `frame.set_cursor_position`, `CursorShape::SteadyUnderScore` on both Dom and WebGl2, `try_borrow()` fallbacks |
| `website` | Ratzilla's own landing page | `Hyperlink`, `utils::open_url` (new tab vs same tab), tachyonfx `hsl_shift` + `RepeatMode::Forever`, flexbox-centered layout |
| `world_map` | World map as Ratatui `canvas::Map` widget | `canvas::Map`, `MapResolution::High`, `Marker::HalfBlock` |

## Best starting points by goal

- **"Just make it work in the browser"** → `minimal` or `animations`
  (smallest surface, Dom/Canvas only).
- **Ships to production** → `website` (effects, hyperlinks, key shortcuts,
  flexbox layout).
- **Performance matters** → `canvas_stress_test` (GPU perf + GC hygiene),
  `demo` (`disable_auto_css_resize` so CSS controls the canvas).
- **Text input from a real editor** → `text_area` (remember the git patch).
- **Clipboard / async JS** → `clipboard`.
- **Pixel-facing layout (HiDPI, exact cell sizes)** → `canvas_waves`
  (`CellSized::cell_size_px` / `cell_size_css_px` usage idea).

## Conventions worth copying

- Every example that wires events sets
  `std::panic::set_hook(Box::new(console_error_panic_hook::hook))` as the
  first line of `main()`, except `minimal`, `world_map`, `text_area`,
  `user_input`, and `unicode`.
- State is `Rc<RefCell<T>>`; the same `Rc` is cloned into the `on_key_event`
  and `draw_web` closures.
- `main() -> std::io::Result<()>` returns after wiring — no event loop to
  own.
- Backend config is centralized in a `*BackendOptions` builder passed to
  `MultiBackendBuilder`.

## Running one example

Each example is its own crate with a dedicated `Cargo.toml` and
`index.html`, so serve it straight from its directory:

```shell
cd examples/canvas_waves
trunk serve
```

Workspace deps (`examples/Cargo.toml`) show the ecosystem habits:
`tachyonfx 0.23.0` with `default-features = false, features = ["wasm"]`,
`web-time 1.1.0` (the wasm-safe `Instant`), `palette 0.7.6` for color math,
and `rand 0.9.2`. `Cargo.lock` files are git-ignored per-example
(`examples/.gitignore`).

## `index.html` variations across examples

- The stress/waves examples write the trunk hook with an explicit manifest
  and wasm-opt level: `<link rel="rust" href="Cargo.toml" data-trunk
  data-wasm-opt="4"/>` (`examples/canvas_stress_test/index.html:26-30`).
  The README's minimal shape is the bare `<link data-trunk rel="rust"/>`.
- `examples/demo/index.html` has **no** `data-trunk` link at all — it sizes
  a `canvas` element directly (`width: calc(100vw - 40px); height: 100%`)
  and pairs with `WebGl2BackendOptions::disable_auto_css_resize()` so CSS
  owns the canvas dimensions (options type imports from
  `ratzilla::backend::webgl2`). Copy this pairing when you want external CSS
  to control layout.
- `canvas_waves`/`pong` add a `#container` div and target it via
  `.grid_id("container")` in the backend options instead of rendering into
  `<body>`.