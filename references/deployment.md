# Deployment

The build/serve story is entirely `trunk` + `wasm32-unknown-unknown`; the
output is a static directory — no runtime, no server framework, no SSR.

## Toolchain

```shell
rustup target add wasm32-unknown-unknown
cargo install --locked trunk        # cargo-binstall also works, see below
```

## Serve during development

```shell
trunk serve                     # http://localhost:8080, auto-reloads
```

Trunk reads `index.html` in your crate root, compiles your crate to wasm,
and injects the module where the `<link data-trunk rel="rust"/>` tag sits
(README.md:102-146).

## Build for production

```shell
trunk build --release
```

Produces a self-contained static site under **`dist/`** — ship that
directory to any static host (GitHub Pages, Netlify, S3, nginx, …). The
wasm, JS, and HTML are all in `dist/`.

## The `index.html` contract

Requirements and hooks from the README's minimal example (README.md:102-146):

- `<head>`: `<link data-trunk rel="rust"/>` — required; trunk injects and
  initializes your wasm module here. Optionally the Fira Code CSS
  (`…/fira_code.min.css`) for a monospace TUI font.
- `<body>` (optional): run JS once the app is live via the startup event:
  ```html
  <script type="module">
    window.addEventListener("TrunkApplicationStarted", (_) => {
      // #[wasm_bindgen] functions are now bound to window.wasmBindings.*
      console.log("application initialized");
    });
  </script>
  ```
- CSS that makes the grid display: centered flexbox `body` at
  `100vw`/`100vh`, dark `#121212` background, and
  `pre { font-family: "Fira Code", monospace; font-size: 16px; }`. The
  DomBackend inherits these fonts, so this is what sizes the terminal grid;
  Canvas/WebGL2 skip the `pre` rule but keep the layout.

## Scaffolding a fresh project

```shell
cargo install cargo-generate
cargo generate ratatui/ratzilla
```

The `simple` template (`templates/simple/`) generates a crate with the
`index.html` above (minus the startup listener), a `DomBackend` counter app,
and a `Cargo.toml` pinning `ratzilla = "0.3.1"` and `ratatui 0.30.1`
(`templates/simple/Cargo.toml.liquid:8-11`). It is itself a complete
deployable — `trunk build --release` it and host `dist/`.

## CI build script

The README documents this CI build flow (README.md:184-204) — it installs
the wasm target and trunk via `cargo-binstall` (musl build):

```bash
#!/bin/bash
set -euo pipefail
export HOME=/root

curl -fsSL https://sh.rustup.rs | sh -s -- -y -t wasm32-unknown-unknown --profile minimal
source "$HOME/.cargo/env"

curl -L --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
  && cargo binstall --targets x86_64-unknown-linux-musl -y trunk

trunk build --release
```

## Deploy templates

- **Vercel**: official template at
  https://vercel.com/templates/other/ratzilla (README.md:206-208).
- **GitHub Pages**: the repo site and example previews all live at
  `https://ratatui.github.io/ratzilla/<example>` — build to `dist/` and point
  CI at a `gh-pages` branch or public folder.

## Gotchas

- In CI, manage the wasm target in your workflow to avoid the interactive
  `rustup` step.
- Static hosting does no URL rewriting/routing — there is no route handler
  to configure.