# Events and widgets

Types live in `src/event.rs`; input arrives through the `WebEventHandler`
trait (`src/render.rs`), which every backend implements and `Terminal` also
exposes (`Terminal<T>: WebRenderer`, `src/render.rs:63-65`).

## Wiring input

Call these once in `main()` before `draw_web`; re-calling an `on_*`
auto-cleans the previous listeners, and dropping the handler removes them.

- `terminal.on_key_event(|KeyEvent| ...)` and
  `terminal.on_mouse_event(|MouseEvent| ...)` return `Result<(), Error>`.
- The four methods: `on_key_event` / `on_mouse_event` / `clear_key_events` /
  `clear_mouse_events` (`src/render.rs:125-173`).
- Closures are `FnMut(..) + 'static` — clone an `Rc<RefCell<App>>` into any
  handler that touches shared state (SKILL.md §3).

## Key events

```rust
pub struct KeyEvent {
    pub code:  KeyCode,
    pub ctrl:  bool,
    pub alt:   bool,
    pub shift: bool,
}
```

`KeyCode` variants (`src/event.rs:51-85`): `Char(char)`, `F(u8)`, `Backspace`,
`Enter`, `Left`, `Right`, `Up`, `Down`, `Tab`, `Delete`, `Home`, `End`,
`PageUp`, `PageDown`, `Esc`, `Unidentified`.

`KeyCode` conversion is strict (`src/event.rs:88-128`): a single-character
`event.key()` maps to `Char(c)`; multi-character keys map only through an
exact name table (`"Enter"`, `"ArrowLeft"`, `"F1"`..`"F12"`); all else is
`Unidentified`. Porting from Crossterm, match exhaustively or you silently
drop input (`examples/text_area`). Common patterns: `Char(' ')`,
`Char('c') if key_event.ctrl`, `Char('e')`, arrows, `Esc`, `Enter`.

## Mouse events

```rust
pub struct MouseEvent {
    pub kind:  MouseEventKind,
    pub col:   u16,   // terminal grid column, not pixels
    pub row:   u16,   // terminal grid row, not pixels
    pub ctrl:  bool,
    pub alt:   bool,
    pub shift: bool,
}
```

`MouseEventKind` (`src/event.rs:148-166`): `Moved`, `ButtonDown(MouseButton)`,
`ButtonUp(MouseButton)`, `SingleClick(MouseButton)`, `DoubleClick(MouseButton)`,
`Entered`, `Exited`, `Unidentified`. `MouseButton`: `Left`, `Right`, `Middle`,
`Back`, `Forward`, `Unidentified` (browser button index 0/1/2/3/4).

Coordinates already translate to the terminal grid (origin top-left).
Handle kinds like `examples/minimal`:

```rust
terminal.on_mouse_event(move |mouse_event| {
    match mouse_event.kind {
        MouseEventKind::ButtonDown(btn) => { /* btn: MouseButton */ }
        MouseEventKind::Moved => { /* mouse_event.col, mouse_event.row */ }
        _ => {}
    }
})?;
```

Per-backend mouse support (verbatim from `src/backend/mod.rs:46-54`):

| Event Type    | DomBackend | CanvasBackend | WebGl2Backend |
|---------------|------------|---------------|---------------|
| `Moved`       | ✓          | ✓             | ✓             |
| `ButtonDown`  | ✓          | ✓             | ✓             |
| `ButtonUp`    | ✓          | ✓             | ✓             |
| `SingleClick` | ✓          | ✓             | ✗*            |
| `DoubleClick` | ✓          | ✓             | ✗*            |
| `Entered`     | ✓          | ✓             | ✗*            |
| `Exited`      | ✓          | ✓             | ✗*            |

\* `webgl2.rs` also maps these types (`src/backend/webgl2.rs:949-973`), so
treat the WebGl2 `✗` as "unverified, test it".

Cursor: Dom and WebGl2 also track a grid cursor — see
`frame.set_cursor_position` in `examples/user_input/src/main.rs:204-210`.

## Web-only widget: `Hyperlink`

The one Ratzilla-owned widget (`ratzilla::widgets::Hyperlink`,
`src/widgets/hyperlink.rs`). It wraps a `Span`, tagging it with
`Modifier::SLOW_BLINK` — the marker bit the **WebGL2 backend scans for**
(`HYPERLINK_MODIFIER`), so it works only there. Configure clicks via
`WebGl2BackendOptions::enable_hyperlinks()` or `.on_hyperlink_click(...)`
(import from `ratzilla::backend::webgl2`, not the root — backends.md §Options).

```rust
// examples/pong/src/main.rs (shape)
use ratzilla::widgets::Hyperlink;

let url = "https://orhun.dev";
let link = Hyperlink::new(url);
let area = Rect::new(right.x, right.y + right.height - 1, url.len() as u16, 1);
f.render_widget(link, area);
```

## Useful `utils` helpers (`src/utils.rs`)

- `set_document_title(&str)` — browser tab title.
- `open_url(url, new_tab: bool)` — navigate, optionally new tab.
- `is_mobile()` — UA-sniffing mobile/tablet check.
- `get_window_size()` / `get_screen_size()` — approximate (÷10/÷20; prefer
  `Terminal::size()`).
- `call_js_function(name, args)` / `call_js_function_with_context(name, this, args)`.

## Async in handlers

Handlers are synchronous; to await JS promises (clipboard, fetch),
`spawn_local` from inside the handler (`examples/clipboard`):

```rust
terminal.on_key_event(move |key_event| {
    let event_state = event_state.clone();
    wasm_bindgen_futures::spawn_local(
        async move { event_state.handle_events(key_event).await },
    );
})?;
```