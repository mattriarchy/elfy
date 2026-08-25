use crate::app::{App, Focus, MenuId, Prompt};
use crate::wrap::{visual_pos, wrap_doc};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let p = app.palette();
    let area = frame.area();
    frame.render_widget(
        Block::default().style(Style::default().bg(p.bg).fg(p.fg)),
        area,
    );

    let vertical = if app.help_visible {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(1)])
            .split(area)
    };

    let main = vertical[0];
    let status = vertical[1];

    // Always three columns. Nothing hides.
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18),
            Constraint::Min(20),
            Constraint::Length(28),
        ])
        .split(main);

    draw_menu(frame, app, cols[0]);
    draw_workspace(frame, app, cols[1]);
    draw_buddy(frame, app, cols[2]);

    draw_status(frame, app, status);
    if app.help_visible {
        if let Some(help) = vertical.get(2) {
            draw_help(frame, app, *help);
        }
    }

    if app.prompt.is_some() {
        draw_prompt(frame, app, area);
    }
}

fn draw_menu(frame: &mut Frame, app: &App, area: Rect) {
    let p = app.palette();
    let vis = app.visible_menus();
    let review_n = app.library.list_review().len();
    let mut lines: Vec<Line> = vec![Line::from(Span::styled(
        "ELFY",
        Style::default().fg(p.muted),
    ))];
    for (i, id) in vis.iter().enumerate() {
        let selected = i == app.menu_index && app.focus == Focus::Menu;
        let current = *id == app.menu_id;
        let marker = if current { "▸ " } else { "  " };
        let name = if *id == MenuId::Review && review_n > 0 {
            format!("{} ({})", id.label(), review_n)
        } else {
            id.label().to_string()
        };
        let key = format!(" [{}]", id.key());
        let name_style = if selected {
            Style::default()
                .bg(p.invert_bg)
                .fg(p.invert_fg)
                .add_modifier(Modifier::BOLD)
        } else if current {
            Style::default().fg(p.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(p.fg)
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{marker}{name}"), name_style),
            Span::styled(key, Style::default().fg(p.muted)),
        ]));
    }
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(p.border))
        .style(Style::default().bg(p.bg));
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_buddy(frame: &mut Frame, app: &App, area: Rect) {
    let p = app.palette();
    let on = app.focus == Focus::Buddy;
    let connected = crate::ai::has_auth();
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(if on { p.accent } else { p.border }).bg(p.bg))
        .style(Style::default().bg(p.bg).fg(p.fg))
        .title(if on { " BrainPal ▸ " } else { " BrainPal " });
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Min(4),
        ])
        .split(inner);

    // SETUP is always the top of this pane — not a job in the list.
    let mut setup: Vec<Line> = vec![Line::from(Span::styled(
        " SETUP BRAINPAL ",
        Style::default()
            .bg(p.accent)
            .fg(p.bg)
            .add_modifier(Modifier::BOLD),
    ))];
    if connected {
        let name = crate::auth::connected_agent().unwrap_or_else(|| "Agent".into());
        setup.push(Line::from(Span::styled(
            format!("{name} · OAuth"),
            Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
        )));
        setup.push(Line::from(Span::styled(
            "D disconnect",
            Style::default().fg(p.muted),
        )));
    } else {
        setup.push(Line::from(Span::styled(
            "not connected",
            Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
        )));
        for (i, kind) in crate::auth::AgentKind::ALL.iter().enumerate() {
            let selected = on && i == app.agent_index;
            let marker = if selected { "▸ " } else { "  " };
            let live = if kind.live() { "" } else { " (soon)" };
            let style = if selected {
                Style::default()
                    .bg(p.invert_bg)
                    .fg(p.invert_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(p.fg)
            };
            setup.push(Line::from(Span::styled(
                format!("{marker}{}{live}", kind.label()),
                style,
            )));
        }
        setup.push(Line::from(Span::styled(
            if on {
                "Enter — add this agent"
            } else {
                "F7 then Enter — add Grok"
            },
            Style::default().fg(p.accent),
        )));
        setup.push(Line::from(Span::styled(
            "OAuth · no API key",
            Style::default().fg(p.muted),
        )));
    }
    frame.render_widget(Paragraph::new(setup), chunks[0]);

    let mut jobs: Vec<Line> = vec![Line::from(Span::styled(
        "jobs",
        Style::default().fg(p.muted),
    ))];
    for (i, label) in app.buddy_items.iter().enumerate() {
        let selected = on && connected && i == app.buddy_index;
        let marker = if selected { "▸ " } else { "  " };
        let style = if selected {
            Style::default().fg(p.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(if connected { p.fg } else { p.muted })
        };
        jobs.push(Line::from(Span::styled(format!("{marker}{label}"), style)));
    }
    frame.render_widget(Paragraph::new(jobs), chunks[1]);

    let blurb = if !connected {
        if on {
            "Pick Grok, press Enter.\nBrowser opens OAuth.\nNo API key."
        } else {
            "Not connected.\nF7 focuses this pane.\nEnter adds Grok."
        }
        .to_string()
    } else {
        let job = app
            .buddy_items
            .get(app.buddy_index)
            .map(|s| s.as_str())
            .unwrap_or("");
        job_blurb(job).to_string()
    };
    frame.render_widget(
        Paragraph::new(blurb)
            .style(Style::default().fg(p.muted))
            .wrap(Wrap { trim: true }),
        chunks[2],
    );
}

fn job_blurb(job: &str) -> &'static str {
    match job {
        "Setup BrainPal" => "Setup BrainPal\n\nConnect an agent with OAuth.",
        "Unstuck" => {
            "Unstuck\n\nWrites the next paragraph into the page, in color. Y keep, N toss, Tab another."
        }
        "Verify" | "Verify names" => {
            "Verify\n\nChecks this page against character Truth names."
        }
        "Refine" | "Refine note" => {
            "Refine\n\nTurns a raw note into one clean statement of intent."
        }
        "Another" | "Another take" => {
            "Another\n\nSame beat, different take. Replaces the current proposal."
        }
        "Gist" => "Gist\n\nWrites a short chapter gist for Outline.",
        "Ask" => "Ask\n\nOne question about the open page. Answer lands as a proposal.",
        "Continuity" => "Continuity\n\nFlags contradictions vs Truth / lore.",
        "Disconnect" => "Disconnect\n\nDrop the OAuth session. Setup BrainPal goes idle.",
        _ => "BrainPal job.",
    }
}

fn draw_workspace(frame: &mut Frame, app: &mut App, area: Rect) {
    match app.menu_id {
        MenuId::Books => draw_book_list(frame, app, area),
        MenuId::Write => {
            draw_editor(frame, app, area);
            if app.chapter_picker {
                draw_chapter_picker(frame, app, area);
            }
        }
        MenuId::Outline | MenuId::Timeline => draw_editor(frame, app, area),
        MenuId::Ideas | MenuId::Characters | MenuId::Lore | MenuId::Style => {
            if app.editing_path.is_some() {
                draw_editor(frame, app, area);
            } else {
                draw_doc_list(frame, app, area);
            }
        }
        MenuId::Adventures => {
            if app.editing_path.is_some() {
                draw_editor(frame, app, area);
            } else {
                draw_adventure_list(frame, app, area);
            }
        }
        MenuId::Review => draw_review_list(frame, app, area),
        MenuId::Mechanics => draw_placeholder(frame, app, area, "Mechanics modules (v3)"),
    }
}

fn draw_editor(frame: &mut Frame, app: &mut App, area: Rect) {
    let p = app.palette();
    let title = if let Some(ref path) = app.editing_path {
        path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "page".into())
    } else {
        app.library
            .chapter()
            .map(|c| c.filename.clone())
            .unwrap_or_else(|| "page".into())
    };
    let block = Block::default()
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(p.muted),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(p.border))
        .style(Style::default().bg(p.bg));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<String> = app.textarea.lines().to_vec();
    let width = inner.width.saturating_sub(6) as usize;
    app.wrap_width = width.max(12);
    app.write_height = inner.height.max(1) as usize;
    let rows = wrap_doc(&lines, app.wrap_width);

    // cursor logical → visual
    let (cur_line, cur_col) = {
        let cursor = app.textarea.cursor();
        (cursor.0, cursor.1)
    };
    let (vrow, vcol) = visual_pos(&rows, cur_line, cur_col);

    // visible window
    let height = inner.height as usize;
    let scroll = vrow.saturating_sub(height.saturating_sub(3));
    let visible = &rows[scroll..rows.len().min(scroll + height)];

    let mut text_lines: Vec<Line> = Vec::new();
    for (i, r) in visible.iter().enumerate() {
        let abs_row = scroll + i;
        let is_cursor_row = abs_row == vrow;
        let is_ai = app
            .proposal
            .as_ref()
            .map(|pr| r.logical >= pr.from_line && r.logical < pr.to_line)
            .unwrap_or(false);

        let gutter = format!("{:>4} ", r.logical + 1);
        let mut spans = vec![Span::styled(
            gutter,
            Style::default().fg(p.muted).bg(p.bg),
        )];

        let style = if is_ai {
            Style::default().fg(p.ai).bg(p.bg)
        } else if is_cursor_row {
            Style::default().fg(p.fg).bg(p.bg)
        } else {
            Style::default().fg(p.fg).bg(p.bg)
        };

        // simple cursor underline on the visual cell
        if is_cursor_row && app.focus == Focus::Workspace {
            let before: String = r.text.chars().take(vcol).collect();
            let mid: String = r.text.chars().skip(vcol).take(1).collect();
            let after: String = r.text.chars().skip(vcol + 1).collect();
            spans.push(Span::styled(before, style));
            spans.push(Span::styled(
                if mid.is_empty() { " ".to_string() } else { mid },
                Style::default().bg(p.accent).fg(p.bg),
            ));
            spans.push(Span::styled(after, style));
        } else {
            spans.push(Span::styled(r.text.clone(), style));
        }
        text_lines.push(Line::from(spans));
    }

    let para = Paragraph::new(text_lines).style(Style::default().bg(p.bg));
    frame.render_widget(para, inner);
}

fn draw_book_list(frame: &mut Frame, app: &App, area: Rect) {
    let p = app.palette();
    let items: Vec<ListItem> = app
        .library
        .books
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let selected = i == app.list_index;
            let marker = if i == app.library.active_book {
                "▸ "
            } else {
                "  "
            };
            let n = b.chapters.len();
            let ch = if n == 1 { "chapter" } else { "chapters" };
            let label = format!("{}{} · {n} {ch}", marker, b.title);
            let style = if selected && app.focus == Focus::Workspace {
                Style::default()
                    .bg(p.invert_bg)
                    .fg(p.invert_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(p.fg).bg(p.bg)
            };
            ListItem::new(Line::from(Span::styled(label, style)))
        })
        .collect();

    let block = Block::default()
        .title(Span::styled(" Books ", Style::default().fg(p.accent)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(p.border))
        .style(Style::default().bg(p.bg));
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn draw_doc_list(frame: &mut Frame, app: &App, area: Rect) {
    let p = app.palette();
    let folder = match app.menu_id.folder() {
        Some(f) => f,
        None => return,
    };
    let docs = app.library.list_docs(folder);
    let title = format!(" {} ", app.menu_id.label());

    let items: Vec<ListItem> = if docs.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  (empty — press N to create)",
            Style::default().fg(p.muted),
        )))]
    } else {
        docs.iter()
            .enumerate()
            .map(|(i, d)| {
                let selected = i == app.list_index && app.focus == Focus::Workspace;
                let marker = if selected { "▸ " } else { "  " };
                let style = if selected {
                    Style::default()
                        .bg(p.invert_bg)
                        .fg(p.invert_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(p.fg).bg(p.bg)
                };
                ListItem::new(Line::from(Span::styled(
                    format!("{marker}{}", d.name),
                    style,
                )))
            })
            .collect()
    };

    let block = Block::default()
        .title(Span::styled(title, Style::default().fg(p.accent)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(p.border))
        .style(Style::default().bg(p.bg));
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn draw_placeholder(frame: &mut Frame, app: &App, area: Rect, msg: &str) {
    let p = app.palette();
    let para = Paragraph::new(msg)
        .style(Style::default().fg(p.muted).bg(p.bg))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(p.border)),
        );
    frame.render_widget(para, area);
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let p = app.palette();
    let book = app
        .library
        .book()
        .map(|b| b.title.as_str())
        .unwrap_or("—");
    let file = if let Some(ref path) = app.editing_path {
        path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
    } else {
        app.library
            .chapter()
            .map(|c| c.filename.clone())
            .unwrap_or_else(|| "—".into())
    };
    let dirty = if app.dirty { " ●" } else { "" };
    let agent = if crate::ai::has_auth() {
        crate::auth::connected_agent().unwrap_or_else(|| "on".into())
    } else {
        "off".into()
    };
    let words = app
        .textarea
        .lines()
        .iter()
        .map(|l| l.split_whitespace().count())
        .sum::<usize>();
    let left = format!(" {book} · {file}{dirty} · BrainPal:{agent} ");
    let right = format!(" {words} words · {} ", app.status_msg);
    let pad = area
        .width
        .saturating_sub((left.len() + right.len()) as u16) as usize;
    let spans = vec![
        Span::styled(left, Style::default().fg(p.status_fg).bg(p.status_bg)),
        Span::styled(" ".repeat(pad), Style::default().bg(p.status_bg)),
        Span::styled(right, Style::default().fg(p.muted).bg(p.status_bg)),
    ];
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    let p = app.palette();
    let line = app.help_line();
    frame.render_widget(
        Paragraph::new(line).style(Style::default().fg(p.help_fg).bg(p.help_bg)),
        area,
    );
}

fn draw_prompt(frame: &mut Frame, app: &App, area: Rect) {
    let p = app.palette();
    let Some(prompt) = &app.prompt else { return };

    let (title, body): (String, String) = match prompt {
        Prompt::NewTitle { buf } => ("New book — title".into(), format!("> {buf}_")),
        Prompt::RenameTitle { buf, .. } => ("Rename book".into(), format!("> {buf}_")),
        Prompt::DeletePin { name, code, buf, .. } => (
            format!("Delete “{name}”?"),
            format!("Type {code} to confirm\n> {buf}_"),
        ),
        Prompt::NewDoc { folder, buf } => {
            let label = match folder {
                crate::library::Folder::Characters => "character",
                crate::library::Folder::Ideas => "idea",
                crate::library::Folder::Lore => "lore",
                crate::library::Folder::Style => "style note",
            };
            (format!("New {label}"), format!("> {buf}_"))
        }
        Prompt::OAuthDevice {
            verification_uri,
            user_code,
            ..
        } => (
            " Connect Grok ".into(),
            format!(
                "A browser tab should be open now.\nIf not, press Enter to open it.\n\nCode is on the clipboard too.\nBackup: {user_code}\n\nB / Enter  open browser\nEsc  cancel"
            ),
        ),
    };

    // center a dialog
    let w = 72u16.min(area.width.saturating_sub(4)).max(40);
    let h = 14u16.min(area.height.saturating_sub(4)).max(10);
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let dialog = Rect::new(x, y, w, h);

    frame.render_widget(Clear, dialog);
    let block = Block::default()
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(p.accent),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(p.accent))
        .style(Style::default().bg(p.status_bg).fg(p.fg));
    let inner = block.inner(dialog);
    frame.render_widget(block, dialog);
    frame.render_widget(
        Paragraph::new(body)
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(p.fg).bg(p.status_bg)),
        inner,
    );
}

fn draw_review_list(frame: &mut Frame, app: &App, area: Rect) {
    let p = app.palette();
    let items = app.library.list_review();
    if items.is_empty() {
        draw_placeholder(frame, app, area, "Review queue empty — no pending items");
        return;
    }
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let selected = i == app.review_index;
            let kind = match item.kind {
                crate::library::ReviewKind::Continuity => "C",
                crate::library::ReviewKind::Outline => "O",
                crate::library::ReviewKind::Refiner => "R",
                crate::library::ReviewKind::Other => "·",
            };
            let label = format!("[{kind}] {}", item.title);
            let style = if selected {
                Style::default()
                    .bg(p.invert_bg)
                    .fg(p.invert_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(p.fg).bg(p.bg)
            };
            ListItem::new(Line::from(Span::styled(label, style)))
        })
        .collect();
    let block = Block::default()
        .title(Span::styled(
            format!(" Review ({}) ", items.len()),
            Style::default().fg(p.accent),
        ))
        .borders(Borders::NONE)
        .style(Style::default().bg(p.bg));
    frame.render_widget(List::new(list_items).block(block), area);
}

fn draw_adventure_list(frame: &mut Frame, app: &App, area: Rect) {
    let p = app.palette();
    let docs = app.library.list_adventures();
    if docs.is_empty() {
        draw_placeholder(frame, app, area, "No adventures yet — add under rpg/adventures/");
        return;
    }
    let list_items: Vec<ListItem> = docs
        .iter()
        .enumerate()
        .map(|(i, doc)| {
            let selected = i == app.list_index;
            let style = if selected {
                Style::default()
                    .bg(p.invert_bg)
                    .fg(p.invert_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(p.fg).bg(p.bg)
            };
            ListItem::new(Line::from(Span::styled(doc.name.clone(), style)))
        })
        .collect();
    let block = Block::default()
        .title(Span::styled(" Adventures ", Style::default().fg(p.accent)))
        .borders(Borders::NONE)
        .style(Style::default().bg(p.bg));
    frame.render_widget(List::new(list_items).block(block), area);
}

fn draw_chapter_picker(frame: &mut Frame, app: &App, area: Rect) {
    let p = app.palette();
    let Some(book) = app.library.book() else { return };
    let w = 48u16.min(area.width.saturating_sub(4));
    let h = ((book.chapters.len() as u16) + 4).min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let dialog = Rect::new(x, y, w, h);

    frame.render_widget(Clear, dialog);
    let block = Block::default()
        .title(Span::styled(
            " Chapter (F3 / Enter) ",
            Style::default().fg(p.accent),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(p.accent))
        .style(Style::default().bg(p.status_bg).fg(p.fg));
    let inner = block.inner(dialog);
    frame.render_widget(block, dialog);

    let items: Vec<ListItem> = book
        .chapters
        .iter()
        .enumerate()
        .map(|(i, ch)| {
            let selected = i == app.list_index;
            let marker = if i == app.library.active_chapter {
                "● "
            } else {
                "  "
            };
            let style = if selected {
                Style::default()
                    .bg(p.invert_bg)
                    .fg(p.invert_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(p.fg).bg(p.status_bg)
            };
            ListItem::new(Line::from(Span::styled(
                format!("{marker}{}. {}", i + 1, ch.title),
                style,
            )))
        })
        .collect();
    frame.render_widget(List::new(items), inner);
}
