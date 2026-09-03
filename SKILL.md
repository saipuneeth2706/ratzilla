---
name: ratzilla
description: Build terminal-styled web apps and browser TUIs with Ratatui compiled to Rust+WebAssembly via Ratzilla, Ratatui's browser backend. Use whenever the user wants to run a terminal app in the browser, port an existing Ratatui/Crossterm TUI to the web, decide between the DOM, Canvas 2D, or GPU-accelerated WebGL2 backends, wire up trunk + wasm32-unknown-unknown builds, or translate browser keyboard/mouse events into TUI terms. Concrete triggers: "can I run my terminal app in the browser?", "run ratatui in a browser", "TUI in a web page", "wasm terminal app", "ratzilla", "browser backend for ratatui", "trunk serve a ratatui app", "terminal look website with Rust", "webgl tui", "dom vs canvas vs webgl2 backend".
---

# Ratzilla

Web/UI reference: this skill is written against the local Ratzilla clone
(v0.3.1, ratatui 0.30.1). Snippets below mirror real repo code; deeper
reference files live in `references/`.

## 1. What Ratzilla is

Ratzilla is the browser backend for [Ratatui]: you compile your normal
Ratatui app to `wasm32-unknown-unknown` and it renders into the DOM instead
of a terminal emulator. You keep `Terminal`, `Frame`, widgets, and layout
exactly as they are — the only things that change are (a) how the app gets
its backend, (b) how the render loop is driven, and (c) how input arrives.
The crate re-exports `ratatui` and `web_sys` wholesale, so all imports stay
under `ratzilla::ratatui::...` and `ratzilla::web_sys::...` (`src/lib.rs`).
Two traits you've never seen in a terminal app appear: `WebRenderer`
(render + input wiring) and `WebEventHandler` (input per backend).

## 2. Quickstart

Two equivalent paths; both end with the same two commands.

**Prerequisites** (once per machine):

```shell
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

**Path A — scaffold** (gives you a working `DomBackend` counter app):

```shell
cargo install cargo-generate
cargo generate ratatui/ratzilla        # answers project-name prompt
cd <project-name>
trunk serve                            # http://localhost:8080
```

**Path B — manual:**

```shell
cargo add ratzilla
```

`Cargo.toml` needs wasm + trunk to know how to build: a plain **bin crate**
(`src/main.rs`) needs no special manifest section — trunk compiles it to wasm
directly, and the `simple` template's `Cargo.toml` has none. Only add
`[lib] crate-type = ["cdylib"]` when you have a real `src/lib.rs`
(`#[wasm_bindgen]` export surface); do not add a `[lib]` section to a
bin-only crate or the manifest fails to parse. The minimal app
(`README.md`) looks like this:

```rust
use std::{cell::RefCell, io, rc::Rc};
use ratzilla::ratatui::{
    layout::Alignment,
    style::Color,
    widgets::{Block, Paragraph},
    Terminal,
};
use ratzilla::{event::KeyCode, DomBackend, WebRenderer};

fn main() -> io::Result<()> {
    let counter = Rc::new(RefCell::new(0));
    let backend = DomBackend::new()?;
    let mut terminal = Terminal::new(backend)?;

    terminal.on_key_event({
        let counter_cloned = counter.clone();
        move |key_event| {
            if key_event.code == KeyCode::Char(' ') {
                *counter_cloned.borrow_mut() += 1;
            }
        }
    })?;

    terminal.draw_web(move |f| {
        let counter = counter.borrow();
        f.render_widget(
            Paragraph::new(format!("Count: {counter}"))
                .alignment(Alignment::Center)
                .block(
                    Block::bordered()
                        .title("Ratzilla")
                        .title_alignment(Alignment::Center)
                        .border_style(Color::Yellow),
                ),
            f.area(),
        );
    });

    Ok(())
}
```

Alongside `main.rs`, drop an `index.html` in the crate root. Trunk injects
your compiled wasm where you point at it:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/firacode/6.2.0/fira_code.min.css" />
    <link data-trunk rel="rust"/>   <!-- trunk builds & injects the wasm here -->
    <title>Ratzilla</title>
  </head>
  <body>
    <script type="module">
      window.addEventListener("TrunkApplicationStarted", (_) => {
        console.log("application initialized");  // optional startup hook
      });
    </script>
  </body>
</html>
```

Style the page so the TUI has somewhere to live: the repo's pages use a
flexbox-centered `body` (100vw/100vh), near-black `#121212` background, and
`pre { font-family: "Fira Code", monospace; }` — the DomBackend inherits the
page font, so the right font-family here is what makes the grid line up.

Serve or build for deploy:

```shell
trunk serve              # dev server, http://localhost:8080
trunk build --release    # bundles everything into ./dist for static hosting
```

## 3. The render loop — the part that breaks terminal-app instincts

A normal Ratatui app owns the loop: `loop { terminal.draw(...); event::read() }`.
Ratzilla does not. `draw_web` registers a `requestAnimationFrame` callback
once; the browser calls that callback ahead of every repaint; the callback
runs `terminal.draw(render_callback)` and then re-schedules itself for the
next frame (`src/render.rs:71-82`). Control flow lives in the browser, not
your stack.

Consequence one: **your render closure is `'static`.** Across the
WASM/JS boundary the closure outlives `main`, so it cannot borrow `&mut`
state you own. `draw_web`'s signature forces this: `F: FnMut(&mut Frame) +
'static` (`src/render.rs:21`).

Consequence two: the same holds for input. Keyboard and mouse arrive through
separate `'static` closures you register with `on_key_event` /
`on_mouse_event` (`src/render.rs:33,46`), not through a blocking `read()`
you call from inside the loop.

Consequence three: your state must be shared by value between all three
closures. The repo-wide idiom is `Rc<RefCell<T>>`, cloned into each `move`
closure. Following the template's shape (`templates/simple/src/main.rs`):

```rust
let state = Rc::new(App::default());

let event_state = Rc::clone(&state);
terminal.on_key_event(move |key_event| event_state.handle_events(key_event))?;

let render_state = Rc::clone(&state);
terminal.draw_web(move |frame| render_state.render(frame));
```

`main()` wires this up, returns `io::Result<()>`, and the process "ends" —
the browser keeps calling the rAF callback forever. There is no per-frame
polling, no `?` back to a caller, no teardown on exit (see pitfalls).

If you need per-frame elapsed time (animation), compute it from
`web_time::Instant::now()` inside the draw closure (`examples/canvas_waves`).

## 4. Choosing a backend

Pick from three, all implementing Ratatui's `Backend`:

- **`WebGl2Backend`** — GPU-accelerated via the beamterm renderer. Pick this
  by default for anything interactive or fullscreen; it's the only one that
  hits 60 fps on large terminals, the only one with hyperlinks, and the
  lightest on memory. Needs a WebGL2-capable browser (modern, 2017+).
- **`CanvasBackend`** — Canvas 2D drawing. Pick when you must target
  non-WebGL2 browsers but still want a canvas. No hyperlinks or text
  selection; emoji only render correctly when they fit in one cell.
- **`DomBackend`** — renders cells as real HTML `<span>`/`<pre>` elements.
  Pick when you want accessibility or CSS-driven styling (it inherits the
  page's font). Slowest for large terminals.

The trade-off table, the per-backend `*Options` builders (cursor shape,
mouse selection mode, font atlas, debug APIs), and the
`MultiBackendBuilder` pattern that real examples use are in
[`references/backends.md`](references/backends.md).

## 5. Events and widgets

Input arrives through `WebEventHandler` methods on the backend (also exposed
via `terminal.on_key_event` / `on_mouse_event` since `Terminal` implements
`WebRenderer`). `KeyEvent { code, ctrl, alt, shift }` and `MouseEvent {
kind, col, row, ctrl, alt, shift }` (grid coords, not pixels). Mouse support
varies by backend. The only web-only widget is `Hyperlink`, which works only
on `WebGl2Backend`. Full shapes, the per-backend mouse matrix, and a usage
snippet: [`references/events-and-widgets.md`](references/events-and-widgets.md).

## 6. Deployment

`trunk build --release` produces a static site in `./dist/` — serve it from
any static host, no server-side setup. The repo documents a Vercel deploy
template and ships a CI build script. The `cargo-generate` scaffold is itself
a deployable project. Details:
[`references/deployment.md`](references/deployment.md).

## 7. Common pitfalls when porting a terminal app

- **No threads, no blocking I/O.** Not one example spawns a thread; state is
  shared via `Rc<RefCell>` or `thread_local!` (`examples/shared/src/fps.rs`).
  On `wasm32-unknown-unknown` there is no thread stack and no blocking socket
  — treat `std::thread::spawn`, `thread::sleep`, and blocking `Read` as
  out of bounds. For async work (e.g. clipboard), use
  `wasm_bindgen_futures::spawn_local` (`examples/clipboard`).
- **A panicking `unwrap()` vanishes into the browser console.** Your app is
  running as JS in a tab; the only place a panic shows up is DevTools, not a
  terminal. Almost every example sets
  `std::panic::set_hook(Box::new(console_error_panic_hook::hook))` first —
  do the same; it at least turns the panic into a readable console trace.
- **The app never exits.** There is no OS process to return control to, no
  alternate screen to restore, no Ctrl+C teardown like
  `terminal.show_cursor()` + restore. `main()` returns `Ok(())` immediately
  after wiring; teardown simply isn't a concept. Don't write cleanup that
  assumes it will run at "exit".
- **Don't call `Terminal::window_size()` on the Canvas backend.**
  `CanvasBackend::window_size()` is `unimplemented!()` and panics
  (`src/backend/canvas.rs:543-545`); Dom and WebGl2 implement it. Use
  `terminal.size()` when you need dimensions portably.
- **`get_window_size()`/`get_screen_size()` are approximate.** The `utils`
  helpers divide raw pixels by hardcoded 10×20 (`src/utils.rs:48-49`) — fine
  for heuristics, but use `Terminal::size()`/`window_size()` for real layout.
- **WebGL2 key events are silently best-effort.** `WebEventHandler::on_key_event`
  is documented as possibly "silently succeed[ing] without registering any
  handlers" (`src/render.rs:156-158`); the current WebGl2 impl does
  register `keydown` on a `tabindex="0"` canvas, but don't build hard
  dependencies on it — and remember the canvas/grid needs focus for keys to
  arrive at all.

## 8. Where to look next

Before writing anything from scratch, look at `examples/` in the repo —
it's a catalog of working patterns, one per directory, with the interesting
API jumps already extracted in
[`references/examples-catalog.md`](references/examples-catalog.md). Find the
example closest to what you're building (input-heavy, animated, canvas-drawn,
clipboard, text input, full demo), copy its skeleton, and adapt.

[Ratatui]: https://ratatui.rs