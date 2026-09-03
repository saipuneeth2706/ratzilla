use std::{cell::RefCell, io, rc::Rc, time::Duration};

mod critical_section_impl;

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
    Frame, Terminal,
};
use ratzilla::{
    backend::webgl2::WebGl2BackendOptions,
    event::{KeyCode, KeyEvent, MouseButton, MouseEventKind},
    utils, WebGl2Backend, WebRenderer,
};
use tachyonfx::{fx, Effect, EffectRenderer, Interpolation};
use web_time::Instant;

const COMMAND: &str = "npx skills add saipuneeth2706/ratzilla";
const PORTFOLIO_URL: &str = "https://www.saipuneeth.me";
const TWITTER_URL: &str = "https://x.com/rsaipuneeth";
const LINK1: &str = "saipuneeth.me";
const LINK2: &str = "@rsaipuneeth";
const BG_COLOR: Color = Color::Rgb(0x0B, 0x09, 0x09);

fn main() -> io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let backend = WebGl2Backend::new_with_options(
        WebGl2BackendOptions::new()
            .grid_id("grid")
            .font_atlas_config(ratzilla::backend::webgl2::FontAtlasConfig::dynamic(
                &["Fira Code"],
                16.0,
            ))
            .cursor_shape(ratzilla::backend::cursor::CursorShape::None)
            .canvas_padding_color(BG_COLOR)
            .enable_hyperlinks(),
    )?;
    let mut terminal = Terminal::new(backend)?;

    let _ = utils::set_document_title("ratzilla");

    let state = Rc::new(App::default());

    let event_state = Rc::clone(&state);
    let _ = terminal.on_key_event(move |key_event| event_state.handle_key_event(key_event));

    let mouse_state = Rc::clone(&state);
    let _ =
        terminal.on_mouse_event(move |mouse_event| mouse_state.handle_mouse_event(mouse_event));

    let render_state = Rc::clone(&state);
    terminal.draw_web(move |frame| render_state.render(frame));

    Ok(())
}

struct App {
    copy_feedback: RefCell<Option<Instant>>,
    animate_copy: RefCell<bool>,
    loaded: RefCell<bool>,
    load_effect: RefCell<Option<Effect>>,
    copy_effect: RefCell<Option<Effect>>,
    last_tick: RefCell<Instant>,
    regions: RefCell<Vec<ClickableRegion>>,
}

struct ClickableRegion {
    rect: Rect,
    action: Action,
}

enum Action {
    CopyCommand,
    OpenUrl(String),
}

impl Default for App {
    fn default() -> Self {
        Self {
            copy_feedback: RefCell::new(None),
            animate_copy: RefCell::new(false),
            loaded: RefCell::new(false),
            load_effect: RefCell::new(None),
            copy_effect: RefCell::new(None),
            last_tick: RefCell::new(Instant::now()),
            regions: RefCell::new(Vec::new()),
        }
    }
}

impl App {
    fn display_copy_feedback(&self) -> bool {
        self.copy_feedback
            .borrow()
            .map(|t| t.elapsed() < Duration::from_secs(2))
            .unwrap_or(false)
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Fill the whole frame background so the grid matches the canvas
        // padding color (otherwise it defaults to near-black).
        {
            let buffer = frame.buffer_mut();
            for y in 0..area.height {
                for x in 0..area.width {
                    buffer[(x, y)].set_bg(BG_COLOR);
                }
            }
        }

        let margin_h = 2;
        let margin_v = 2;
        let block_area = Rect {
            x: margin_h,
            y: margin_v,
            width: area.width.saturating_sub(margin_h * 2),
            height: area.height.saturating_sub(margin_v * 2),
        };

        let block = Block::bordered()
            .title(" ratzilla ")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(block_area);
        let content_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height,
        };

        // Copy feedback state (needed for sizing the command box).
        let show_copied = self.display_copy_feedback();
        let cmd_label = if show_copied { "Copied!" } else { "[Copy]" };

        // Fixed width for the command box, centered horizontally.
        let cmd_rendered = " $ ".len() as u16 + COMMAND.len() as u16 + 2 + cmd_label.len() as u16;
        let box_width = (cmd_rendered + 2).min(inner.width); // +2 for borders
        let box_x = inner.x + (inner.width.saturating_sub(box_width)) / 2;

        // Vertically center the content within the block's inner area.
        let content_height: u16 = 10;
        let top = inner.y + inner.height.saturating_sub(content_height) / 2;

        let header_y = top;
        let title_y = top + 1;
        let install_y = top + 4;
        let box_y = top + 5;
        let link_y = top + 9;

        // Header / tagline
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "Build terminal-themed web applications with Rust and WebAssembly. Powered by Ratatui.",
                Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD),
            )))
            .alignment(Alignment::Center),
            Rect {
                x: inner.x,
                y: header_y,
                width: inner.width,
                height: 1,
            },
        );

        // Description (two lines)
        let desc_text = vec![
            Line::from(Span::styled(
                "https://ratatui.github.io/ratzilla ",
                Style::default().fg(Color::Green),
            )),
            Line::from(Span::styled(
                "https://github.com/ratatui/ratzilla",
                Style::default().fg(Color::Yellow),
            )),
        ];
        frame.render_widget(
            Paragraph::new(desc_text).alignment(Alignment::Center),
            Rect {
                x: inner.x,
                y: title_y,
                width: inner.width,
                height: 2,
            },
        );

        // Install hint
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "Install the ratzilla skills:",
                Style::default().fg(Color::DarkGray),
            )))
            .alignment(Alignment::Center),
            Rect {
                x: inner.x,
                y: install_y,
                width: inner.width,
                height: 1,
            },
        );

        // Command box
        let cmd_label_style = if show_copied {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Yellow)
        };

        let box_area = Rect {
            x: box_x,
            y: box_y,
            width: box_width,
            height: 3,
        };

        let box_block = Block::bordered()
            .border_type(BorderType::Double)
            .border_style(if show_copied {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            });

        let cmd_text = Line::from(vec![
            Span::styled(" $ ", Style::default().fg(Color::DarkGray)),
            Span::styled(COMMAND, Style::default().fg(Color::LightCyan)),
            Span::raw("  "),
            Span::styled(cmd_label, cmd_label_style),
        ]);

        frame.render_widget(box_block.clone(), box_area);
        frame.render_widget(
            Paragraph::new(cmd_text).alignment(Alignment::Left),
            box_block.inner(box_area),
        );

        // Links
        let links = Line::from(vec![
            Span::styled(
                LINK1,
                Style::default().fg(Color::Rgb(0x76, 0x92, 0xFF)).add_modifier(Modifier::UNDERLINED),
            ),
            Span::raw("   "),
            Span::styled(
                LINK2,
                Style::default().fg(Color::Rgb(0x76, 0x92, 0xFF)).add_modifier(Modifier::UNDERLINED),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(links).alignment(Alignment::Center),
            Rect {
                x: inner.x,
                y: link_y,
                width: inner.width,
                height: 1,
            },
        );

        // Record clickable regions.
        let mut regions = self.regions.borrow_mut();
        regions.clear();

        let box_content_x = box_x + 1;
        let copy_x = box_content_x + cmd_rendered - cmd_label.len() as u16;
        regions.push(ClickableRegion {
            rect: Rect {
                x: copy_x,
                y: box_y + 1,
                width: cmd_label.len() as u16,
                height: 1,
            },
            action: Action::CopyCommand,
        });

        let total_links_len = LINK1.len() as u16 + 3 + LINK2.len() as u16;
        let links_start_x = inner.x + (inner.width.saturating_sub(total_links_len)) / 2;
        regions.push(ClickableRegion {
            rect: Rect {
                x: links_start_x,
                y: link_y,
                width: LINK1.len() as u16,
                height: 1,
            },
            action: Action::OpenUrl(PORTFOLIO_URL.to_string()),
        });
        regions.push(ClickableRegion {
            rect: Rect {
                x: links_start_x + LINK1.len() as u16 + 3,
                y: link_y,
                width: LINK2.len() as u16,
                height: 1,
            },
            action: Action::OpenUrl(TWITTER_URL.to_string()),
        });

        // ---- Micro-animations ----

        // Per-frame elapsed time.
        let now = Instant::now();
        let elapsed: tachyonfx::Duration = now.duration_since(*self.last_tick.borrow()).into();
        *self.last_tick.borrow_mut() = now;

        // Page-load reveal: coalesce the content in once, subtly.
        let mut loaded = self.loaded.borrow_mut();
        if !*loaded {
            *self.load_effect.borrow_mut() = Some(fx::coalesce((
                1000,
                Interpolation::QuadOut,
            )));
            *loaded = true;
        }
        drop(loaded);

        // Copy trigger: re-reveal the command box when copied.
        if *self.animate_copy.borrow() {
            *self.copy_effect.borrow_mut() = Some(fx::coalesce((
                700,
                Interpolation::QuadOut,
            )));
            *self.animate_copy.borrow_mut() = false;
        }

        // Apply the load effect over the whole content area.
        if self.load_effect.borrow().as_ref().is_some_and(|e| e.running()) {
            let mut effect = self.load_effect.borrow_mut().take().unwrap();
            frame.render_effect(&mut effect, content_area, elapsed);
            if effect.running() {
                *self.load_effect.borrow_mut() = Some(effect);
            }
        }

        // Apply the copy effect over the command box.
        if self.copy_effect.borrow().as_ref().is_some_and(|e| e.running()) {
            let mut effect = self.copy_effect.borrow_mut().take().unwrap();
            frame.render_effect(&mut effect, box_area, elapsed);
            if effect.running() {
                *self.copy_effect.borrow_mut() = Some(effect);
            }
        }
    }

    fn handle_key_event(&self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('c') | KeyCode::Char('C') => self.copy_command(),
            _ => {}
        }
    }

    fn handle_mouse_event(&self, event: ratzilla::event::MouseEvent) {
        if let MouseEventKind::ButtonDown(MouseButton::Left) = event.kind {
            let regions = self.regions.borrow();
            for region in regions.iter() {
                if event.col >= region.rect.x
                    && event.col < region.rect.x + region.rect.width
                    && event.row >= region.rect.y
                    && event.row < region.rect.y + region.rect.height
                {
                    match &region.action {
                        Action::CopyCommand => self.copy_command(),
                        Action::OpenUrl(url) => {
                            let _ = utils::open_url(url, true);
                        }
                    }
                    break;
                }
            }
        }
    }

    fn copy_command(&self) {
        *self.copy_feedback.borrow_mut() = Some(Instant::now());
        *self.animate_copy.borrow_mut() = true;

        wasm_bindgen_futures::spawn_local(async {
            if let Some(window) = web_sys::window() {
                let clipboard = window.navigator().clipboard();
                match clipboard.write_text(COMMAND).await {
                    Ok(_) => web_sys::console::log_1(&"Copied to clipboard".into()),
                    Err(e) => web_sys::console::warn_1(&e.into()),
                }
            }
        });
    }
}
