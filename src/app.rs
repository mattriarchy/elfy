use crate::library::{Folder, Library};
use crate::local_ai;
use crate::wrap;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Style};
use tui_textarea::{CursorMove, Input, Key, TextArea};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuId {
    Books,
    Write,
    Ideas,
    Outline,
    Characters,
    Lore,
    Style,
    Mechanics,
    Adventures,
    Review,
    Timeline,
}

impl MenuId {
    pub const ALL: [MenuId; 9] = [
        MenuId::Books,
        MenuId::Write,
        MenuId::Ideas,
        MenuId::Outline,
        MenuId::Characters,
        MenuId::Lore,
        MenuId::Style,
        MenuId::Review,
        MenuId::Timeline,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MenuId::Books => "Books",
            MenuId::Write => "Write",
            MenuId::Ideas => "Ideas",
            MenuId::Outline => "Outline",
            MenuId::Characters => "Characters",
            MenuId::Lore => "Lore",
            MenuId::Style => "Style",
            MenuId::Mechanics => "Mechanics",
            MenuId::Adventures => "Adventures",
            MenuId::Review => "Review",
            MenuId::Timeline => "Timeline",
        }
    }

    pub fn key(self) -> char {
        match self {
            MenuId::Books => 'B',
            MenuId::Write => 'W',
            MenuId::Ideas => 'I',
            MenuId::Outline => 'O',
            MenuId::Characters => 'C',
            MenuId::Lore => 'L',
            MenuId::Style => 'S',
            MenuId::Mechanics => 'M',
            MenuId::Adventures => 'A',
            MenuId::Review => 'R',
            MenuId::Timeline => 'T',
        }
    }

    pub fn from_letter(c: char) -> Option<MenuId> {
        Self::ALL
            .into_iter()
            .find(|m| m.key() == c.to_ascii_uppercase())
    }

    pub fn folder(self) -> Option<Folder> {
        match self {
            MenuId::Characters => Some(Folder::Characters),
            MenuId::Ideas => Some(Folder::Ideas),
            MenuId::Lore => Some(Folder::Lore),
            MenuId::Style => Some(Folder::Style),
            _ => None,
        }
    }

    pub fn is_rpg_only(self) -> bool {
        matches!(self, MenuId::Mechanics | MenuId::Adventures)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Menu,
    Workspace,
    Buddy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeId {
    Dark,
    WpBlue,
    Green,
    Amber,
}

impl ThemeId {
    pub fn next(self) -> Self {
        match self {
            ThemeId::Dark => ThemeId::WpBlue,
            ThemeId::WpBlue => ThemeId::Green,
            ThemeId::Green => ThemeId::Amber,
            ThemeId::Amber => ThemeId::Dark,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ThemeId::Dark => "dark",
            ThemeId::WpBlue => "WP-blue",
            ThemeId::Green => "green",
            ThemeId::Amber => "amber",
        }
    }

    pub fn palette(self) -> Palette {
        match self {
            ThemeId::Dark => Palette {
                bg: rgb(22, 22, 22),
                fg: rgb(232, 232, 232),
                muted: rgb(150, 150, 150),
                accent: rgb(255, 196, 72),
                border: rgb(90, 90, 90),
                invert_bg: rgb(232, 232, 232),
                invert_fg: rgb(22, 22, 22),
                status_bg: rgb(36, 36, 36),
                status_fg: rgb(232, 232, 232),
                help_bg: rgb(28, 28, 28),
                help_fg: rgb(255, 196, 72),
                ai: rgb(120, 180, 255),
            },
            ThemeId::WpBlue => Palette {
                bg: rgb(0, 0, 170),
                fg: rgb(245, 245, 255),
                muted: rgb(180, 180, 255),
                accent: rgb(255, 255, 102),
                border: rgb(122, 122, 255),
                invert_bg: rgb(245, 245, 255),
                invert_fg: rgb(0, 0, 170),
                status_bg: rgb(245, 245, 255),
                status_fg: rgb(0, 0, 170),
                help_bg: rgb(0, 0, 85),
                help_fg: rgb(255, 255, 102),
                ai: rgb(180, 220, 255),
            },
            ThemeId::Green => Palette {
                bg: rgb(3, 26, 8),
                fg: rgb(124, 255, 154),
                muted: rgb(61, 154, 85),
                accent: rgb(232, 255, 106),
                border: rgb(46, 107, 60),
                invert_bg: rgb(124, 255, 154),
                invert_fg: rgb(3, 26, 8),
                status_bg: rgb(1, 34, 12),
                status_fg: rgb(184, 255, 206),
                help_bg: rgb(1, 20, 8),
                help_fg: rgb(232, 255, 106),
                ai: rgb(140, 220, 180),
            },
            ThemeId::Amber => Palette {
                bg: rgb(26, 16, 0),
                fg: rgb(255, 191, 77),
                muted: rgb(176, 120, 32),
                accent: rgb(255, 244, 163),
                border: rgb(138, 90, 18),
                invert_bg: rgb(255, 191, 77),
                invert_fg: rgb(26, 16, 0),
                status_bg: rgb(18, 11, 0),
                status_fg: rgb(255, 226, 154),
                help_bg: rgb(18, 11, 0),
                help_fg: rgb(255, 244, 163),
                ai: rgb(255, 210, 140),
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub fg: Color,
    pub muted: Color,
    pub accent: Color,
    pub border: Color,
    pub invert_bg: Color,
    pub invert_fg: Color,
    pub status_bg: Color,
    pub status_fg: Color,
    pub help_bg: Color,
    pub help_fg: Color,
    pub ai: Color,
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

pub enum Prompt {
    NewTitle { buf: String },
    RenameTitle { index: usize, buf: String },
    DeletePin {
        index: usize,
        name: String,
        code: String,
        buf: String,
    },
    NewDoc { folder: Folder, buf: String },
    /// OAuth device-code: show URI + user_code; Enter starts poll.
    OAuthDevice {
        verification_uri: String,
        user_code: String,
        device_code: String,
        interval: u64,
        expires_in: u64,
    },
}

/// AI proposal currently sitting in the editor (accent range).
#[derive(Clone)]
pub struct Proposal {
    pub from_line: usize,
    pub to_line: usize, // exclusive
    pub text: String,
}

pub struct App {
    pub library: Library,
    pub menu_index: usize,
    pub menu_id: MenuId,
    pub focus: Focus,
    pub menu_collapsed: bool,
    pub buddy_collapsed: bool,
    pub help_visible: bool,
    pub dirty: bool,
    pub textarea: TextArea<'static>,
    pub status_msg: String,
    pub should_quit: bool,
    pub last_esc: Option<std::time::Instant>,
    pub theme: ThemeId,
    pub prompt: Option<Prompt>,
    /// When editing a SideDoc (Characters / Ideas / Lore / Style)
    pub editing_path: Option<std::path::PathBuf>,
    pub list_index: usize,
    pub proposal: Option<Proposal>,
    pub buddy_items: Vec<String>,
    pub buddy_index: usize,
    /// Review queue selection
    pub review_index: usize,
    /// F3 chapter picker overlay
    pub chapter_picker: bool,
    /// Agent picker when not connected (F6)
    pub picking_agent: bool,
    pub agent_index: usize,
    /// True when focus is in the Setup BrainPal block
    pub in_setup: bool,
    oauth_rx: Option<std::sync::mpsc::Receiver<Result<crate::auth::TokenSet, String>>>,
    /// Last editor wrap width (visual columns). Up/down move by visual rows.
    pub wrap_width: usize,
    pub write_height: usize,
    vis_col: usize,
}

impl App {
    pub fn new(library: Library) -> Self {
        let mut app = Self {
            library,
            menu_index: 1, // Write
            menu_id: MenuId::Write,
            focus: Focus::Workspace,
            menu_collapsed: false,
            buddy_collapsed: false,
            help_visible: true,
            dirty: false,
            textarea: TextArea::default(),
            status_msg: String::new(),
            should_quit: false,
            last_esc: None,
            theme: ThemeId::Dark,
            prompt: None,
            editing_path: None,
            list_index: 0,
            proposal: None,
            buddy_items: default_buddy(),
            buddy_index: 0,
            review_index: 0,
            chapter_picker: false,
            picking_agent: !crate::ai::has_auth(),
            agent_index: 0,
            in_setup: !crate::ai::has_auth(),
            oauth_rx: None,
            wrap_width: 60,
            write_height: 20,
            vis_col: 0,
        };
        app.load_active_into_editor();
        if crate::ai::has_auth() {
            app.status_msg = format!(
                "agent: {}",
                crate::auth::connected_agent().unwrap_or_else(|| "on".into())
            );
        } else {
            app.status_msg = "BrainPal not connected — F7 then Enter to add Grok".into();
        }
        app
    }

    pub fn palette(&self) -> Palette {
        self.theme.palette()
    }

    pub fn visible_menus(&self) -> Vec<MenuId> {
        MenuId::ALL.to_vec()
    }

    pub fn help_line(&self) -> String {
        if let Some(p) = &self.prompt {
            return match p {
                Prompt::NewTitle { .. } => " Type a title   Enter create   Esc cancel ".into(),
                Prompt::RenameTitle { .. } => " Type new title   Enter rename   Esc cancel ".into(),
                Prompt::DeletePin { code, .. } => {
                    format!(" Type {code} to delete   Enter confirm   Esc cancel ")
                }
                Prompt::NewDoc { .. } => " Type name   Enter create   Esc cancel ".into(),
                Prompt::OAuthDevice { .. } => {
                    " Browser opened   Enter reopens   waiting   Esc cancel ".into()
                }
            };
        }
        if self.proposal.is_some() {
            return " Y keep   N toss   Tab next   type = overwrite   Esc cancel proposal ".into();
        }
        match self.focus {
            Focus::Buddy => {
                if !crate::ai::has_auth() {
                    " SETUP BRAINPAL   ↑↓ agent   Enter connect Grok   Esc page ".into()
                } else {
                    " Enter run job   S setup   D disconnect   ↑↓   Esc page ".into()
                }
            }
            Focus::Menu => " Enter open   letter jump   Esc workspace ".into(),
            Focus::Workspace => match self.menu_id {
                MenuId::Books => {
                    " N new   E rename   D delete   Enter open   F7 BrainPal   F10 menu "
                        .into()
                }
                MenuId::Write => {
                    if crate::ai::has_auth() {
                        " F2 save   F3 chapter   F7 BrainPal   F10 menu ".into()
                    } else {
                        " F2 save   F7 BrainPal — Enter adds Grok   F10 menu ".into()
                    }
                }
                MenuId::Ideas | MenuId::Characters | MenuId::Lore | MenuId::Style => {
                    " Enter edit   N new   Esc list   F2 save   F7 BrainPal   F10 menu ".into()
                }
                MenuId::Adventures => {
                    " Enter open   N new   Esc list   F2 save   F10 menu ".into()
                }
                MenuId::Review => {
                    " Y approve   N reject   S skip   ↑↓   Enter open body   F10 menu ".into()
                }
                MenuId::Outline => " (gists from F2)   F10 menu   F8 theme ".into(),
                MenuId::Timeline => " (local scan)   F10 menu   F8 theme ".into(),
                _ => " F10 menu   F8 theme   F7 BrainPal   Ctrl+C quit ".into(),
            },
        }
    }

    fn style_editor(&mut self) {
        let p = self.palette();
        self.textarea.set_style(Style::default().fg(p.fg).bg(p.bg));
        self.textarea
            .set_cursor_style(Style::default().bg(p.accent).fg(p.bg));
        self.textarea
            .set_cursor_line_style(Style::default().bg(p.bg).fg(p.fg));
    }

    pub fn cycle_theme(&mut self) {
        self.theme = self.theme.next();
        self.style_editor();
        self.status_msg = format!("theme: {}", self.theme.name());
    }

    pub fn load_active_into_editor(&mut self) {
        self.proposal = None;
        self.editing_path = None;
        let body = self
            .library
            .chapter()
            .map(|c| c.body.clone())
            .unwrap_or_default();
        let mut ta = TextArea::from(body.lines().map(|l| l.to_string()));
        ta.set_placeholder_text("Empty — start typing, or N on Books to create a book");
        self.textarea = ta;
        self.style_editor();
        self.dirty = false;
        if let Some(ch) = self.library.chapter() {
            self.status_msg = format!("opened {}", ch.filename);
        } else if self.library.is_empty() {
            self.status_msg = "no books — press N to create one".into();
        }
    }

    pub fn load_path_into_editor(&mut self, path: std::path::PathBuf, body: String) {
        self.proposal = None;
        self.editing_path = Some(path);
        let mut ta = TextArea::from(body.lines().map(|l| l.to_string()));
        self.textarea = ta;
        self.style_editor();
        self.dirty = false;
        self.status_msg = "editing side doc".into();
    }

    pub fn sync_editor_to_library(&mut self) {
        let body = self.textarea.lines().join("\n");
        if let Some(ref path) = self.editing_path {
            let _ = self.library.save_path(path, &body);
        } else {
            self.library.set_chapter_body(body);
        }
    }

    pub fn save(&mut self) {
        if let Some(path) = self.editing_path.clone() {
            let body = self.textarea.lines().join("\n");
            match self.library.save_path(&path, &body) {
                Ok(()) => {
                    let _ = self.library.git_commit(&format!("save {}", path.display()));
                    self.dirty = false;
                    self.status_msg = "saved + git".into();
                }
                Err(e) => self.status_msg = format!("save failed: {e}"),
            }
            return;
        }
        self.sync_editor_to_library();
        if self.library.chapter().is_none() {
            self.status_msg = "nothing to save".into();
            return;
        }
        match self.library.save_active() {
            Ok(()) => {
                let _ = self.library.write_gist_active();
                let name = self
                    .library
                    .chapter()
                    .map(|c| c.filename.clone())
                    .unwrap_or_else(|| "chapter".into());
                let _ = self.library.git_commit(&format!("save {name}"));
                self.dirty = false;
                self.status_msg = "saved + gist + git".into();
            }
            Err(e) => self.status_msg = format!("save failed: {e}"),
        }
    }

    fn start_new_book(&mut self) {
        self.prompt = Some(Prompt::NewTitle { buf: String::new() });
    }

    fn start_rename_book(&mut self) {
        if self.library.is_empty() {
            self.status_msg = "no book to rename".into();
            return;
        }
        let index = self.list_index.min(self.library.books.len().saturating_sub(1));
        let buf = self
            .library
            .books
            .get(index)
            .map(|b| b.title.clone())
            .unwrap_or_default();
        self.prompt = Some(Prompt::RenameTitle { index, buf });
    }

    fn start_delete_book(&mut self) {
        if self.library.is_empty() {
            self.status_msg = "no book to delete".into();
            return;
        }
        let index = self.library.active_book;
        let name = self
            .library
            .book()
            .map(|b| b.title.clone())
            .unwrap_or_default();
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let code = format!("{:04}", 1000 + (n % 9000));
        self.prompt = Some(Prompt::DeletePin {
            index,
            name,
            code,
            buf: String::new(),
        });
    }

    fn start_new_doc(&mut self, folder: Folder) {
        self.prompt = Some(Prompt::NewDoc {
            folder,
            buf: String::new(),
        });
    }

    fn handle_prompt(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.prompt = None;
            self.oauth_rx = None;
            self.status_msg = "cancelled".into();
            return;
        }
        match self.prompt.take() {
            Some(Prompt::NewTitle { mut buf }) => match key.code {
                KeyCode::Enter => {
                    let title = buf.trim().to_string();
                    if title.is_empty() {
                        self.prompt = Some(Prompt::NewTitle { buf });
                    } else {
                        self.finish_new_book(title);
                    }
                }
                KeyCode::Backspace => {
                    buf.pop();
                    self.prompt = Some(Prompt::NewTitle { buf });
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if buf.len() < 80 {
                        buf.push(c);
                    }
                    self.prompt = Some(Prompt::NewTitle { buf });
                }
                _ => self.prompt = Some(Prompt::NewTitle { buf }),
            },
            Some(Prompt::RenameTitle { index, mut buf }) => match key.code {
                KeyCode::Enter => {
                    let title = buf.trim().to_string();
                    if title.is_empty() {
                        self.prompt = Some(Prompt::RenameTitle { index, buf });
                    } else {
                        match self.library.rename_book(index, &title) {
                            Ok(()) => {
                                self.status_msg = format!("renamed to {title}");
                                let _ = self.library.git_commit(&format!("rename book to {title}"));
                            }
                            Err(e) => self.status_msg = format!("rename failed: {e}"),
                        }
                    }
                }
                KeyCode::Backspace => {
                    buf.pop();
                    self.prompt = Some(Prompt::RenameTitle { index, buf });
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if buf.len() < 80 {
                        buf.push(c);
                    }
                    self.prompt = Some(Prompt::RenameTitle { index, buf });
                }
                _ => self.prompt = Some(Prompt::RenameTitle { index, buf }),
            },
            Some(Prompt::DeletePin {
                index,
                name,
                code,
                mut buf,
            }) => match key.code {
                KeyCode::Enter => {
                    if buf == code {
                        match self.library.delete_book(index) {
                            Ok(()) => {
                                self.status_msg = format!("deleted {name}");
                                self.load_active_into_editor();
                                self.menu_id = MenuId::Books;
                            }
                            Err(e) => self.status_msg = format!("delete failed: {e}"),
                        }
                    } else {
                        self.status_msg = "wrong code — not deleted".into();
                    }
                }
                KeyCode::Backspace => {
                    buf.pop();
                    self.prompt = Some(Prompt::DeletePin {
                        index,
                        name,
                        code,
                        buf,
                    });
                }
                KeyCode::Char(c) if c.is_ascii_digit() && buf.len() < 4 => {
                    buf.push(c);
                    self.prompt = Some(Prompt::DeletePin {
                        index,
                        name,
                        code,
                        buf,
                    });
                }
                _ => {
                    self.prompt = Some(Prompt::DeletePin {
                        index,
                        name,
                        code,
                        buf,
                    });
                }
            },
            Some(Prompt::NewDoc { folder, mut buf }) => match key.code {
                KeyCode::Enter => {
                    let name = buf.trim().to_string();
                    if name.is_empty() {
                        self.prompt = Some(Prompt::NewDoc { folder, buf });
                    } else {
                        match self.library.create_doc(folder, &name) {
                            Ok(path) => {
                                let body = std::fs::read_to_string(&path).unwrap_or_default();
                                self.load_path_into_editor(path, body);
                                self.status_msg = format!("created {name}");
                            }
                            Err(e) => self.status_msg = format!("create failed: {e}"),
                        }
                    }
                }
                KeyCode::Backspace => {
                    buf.pop();
                    self.prompt = Some(Prompt::NewDoc { folder, buf });
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if buf.len() < 60 {
                        buf.push(c);
                    }
                    self.prompt = Some(Prompt::NewDoc { folder, buf });
                }
                _ => self.prompt = Some(Prompt::NewDoc { folder, buf }),
            },
            Some(Prompt::OAuthDevice {
                verification_uri,
                user_code,
                device_code,
                interval,
                expires_in,
            }) => match key.code {
                KeyCode::Enter
                | KeyCode::Char('b')
                | KeyCode::Char('B')
                | KeyCode::Char('o')
                | KeyCode::Char('O') => {
                    let opened = crate::auth::open_browser(&verification_uri);
                    let _ = crate::auth::copy_clipboard(&verification_uri);
                    self.status_msg = if opened {
                        "browser opened — approve there, this pane waits".into()
                    } else {
                        format!("no browser — URL on clipboard: {verification_uri}")
                    };
                    self.prompt = Some(Prompt::OAuthDevice {
                        verification_uri,
                        user_code,
                        device_code,
                        interval,
                        expires_in,
                    });
                }
                _ => {
                    self.prompt = Some(Prompt::OAuthDevice {
                        verification_uri,
                        user_code,
                        device_code,
                        interval,
                        expires_in,
                    });
                }
            },
            None => {}
        }
    }

    fn finish_new_book(&mut self, title: String) {
        match self.library.create_book(&title) {
            Ok(()) => {
                self.status_msg = format!("created {title}");
                self.menu_id = MenuId::Write;
                self.menu_index = self
                    .visible_menus()
                    .iter()
                    .position(|m| *m == MenuId::Write)
                    .unwrap_or(0);
                self.load_active_into_editor();
            }
            Err(e) => self.status_msg = format!("create failed: {e}"),
        }
    }

    pub fn enter_room(&mut self, id: MenuId) {
        self.menu_id = id;
        self.list_index = 0;
        self.proposal = None;
        self.editing_path = None;
        match id {
            MenuId::Write => self.load_active_into_editor(),
            MenuId::Outline => {
                let text = self.library.outline_text();
                let mut ta = TextArea::from(text.lines().map(|l| l.to_string()));
                ta.set_cursor_style(Style::default());
                self.textarea = ta;
                self.style_editor();
            }
            MenuId::Timeline => {
                let text = self.library.timeline_text();
                let mut ta = TextArea::from(text.lines().map(|l| l.to_string()));
                ta.set_cursor_style(Style::default());
                self.textarea = ta;
                self.style_editor();
            }
            MenuId::Books | MenuId::Review | MenuId::Mechanics | MenuId::Adventures => {
                // list handled in ui
                self.list_index = 0;
                self.review_index = 0;
            }
            _ => {
                // side docs — stay in list until Enter
                self.list_index = 0;
            }
        }
        self.focus = Focus::Workspace;
    }

    /// Insert a proposal at the end of the current document (accent range).
    pub fn insert_proposal(&mut self, text: String) {
        let lines = self.textarea.lines();
        let from = lines.len();
        // ensure a blank line before proposal
        if from > 0 && !lines.last().map(|l| l.is_empty()).unwrap_or(true) {
            self.textarea.insert_str("\n\n");
        } else {
            self.textarea.insert_str("\n");
        }
        let from = self.textarea.lines().len();
        self.textarea.insert_str(&text);
        let to = self.textarea.lines().len();
        self.proposal = Some(Proposal {
            from_line: from,
            to_line: to,
            text,
        });
        self.dirty = true;
        self.status_msg = "proposal in page — Y keep / N toss".into();
        self.focus = Focus::Workspace;
    }

    pub fn keep_proposal(&mut self) {
        self.proposal = None;
        self.status_msg = "kept".into();
    }

    pub fn toss_proposal(&mut self) {
        if let Some(p) = self.proposal.take() {
            // crude: rebuild without the proposal lines
            let lines: Vec<String> = self.textarea.lines().to_vec();
            let keep: Vec<String> = lines
                .into_iter()
                .enumerate()
                .filter(|(i, _)| *i < p.from_line || *i >= p.to_line)
                .map(|(_, l)| l)
                .collect();
            let ta = TextArea::from(keep);
            self.textarea = ta;
            self.style_editor();
            self.dirty = true;
            self.status_msg = "tossed".into();
        }
    }

    fn focus_agent_pane(&mut self) {
        self.buddy_collapsed = false;
        self.buddy_items = default_buddy();
        if self.buddy_index >= self.buddy_items.len() {
            self.buddy_index = 0;
        }
        self.picking_agent = !crate::ai::has_auth();
        self.in_setup = true;
        self.buddy_index = 0;
        self.focus = Focus::Buddy;
        if self.in_setup {
            self.status_msg = "Setup BrainPal — pick agent, Enter to connect".into();
        }
    }

    fn start_selected_agent(&mut self) {
        let kind = crate::auth::AgentKind::ALL
            .get(self.agent_index)
            .copied()
            .unwrap_or(crate::auth::AgentKind::Grok);
        if !kind.live() {
            // Claude uses the same OAuth slot once wired; Grok is the live path.
            self.status_msg =
                "Claude OAuth next — pick Grok (live now) or wait for Claude slot".into();
            return;
        }
        match crate::auth::login() {
            Ok(pending) => {
                let opened = crate::auth::open_browser(&pending.verification_uri);
                let _copied = crate::auth::copy_clipboard(&pending.verification_uri);
                let (tx, rx) = std::sync::mpsc::channel();
                let wait = crate::auth::DevicePending {
                    device_code: pending.device_code.clone(),
                    user_code: pending.user_code.clone(),
                    verification_uri: pending.verification_uri.clone(),
                    expires_in: pending.expires_in,
                    interval: pending.interval,
                };
                std::thread::spawn(move || {
                    let _ = tx.send(wait.wait_for_tokens().map_err(|e| e.to_string()));
                });
                self.oauth_rx = Some(rx);
                self.prompt = Some(Prompt::OAuthDevice {
                    verification_uri: pending.verification_uri,
                    user_code: pending.user_code,
                    device_code: pending.device_code,
                    interval: pending.interval,
                    expires_in: pending.expires_in,
                });
                self.status_msg = if opened {
                    format!("browser opened for {} — approve, this pane waits", kind.label())
                } else {
                    format!("could not open browser — press B  ({})", kind.label())
                };
            }
            Err(e) => self.status_msg = format!("OAuth start failed: {e}"),
        }
    }

    pub fn poll_oauth(&mut self) {
        let Some(rx) = self.oauth_rx.as_ref() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(tokens)) => {
                self.oauth_rx = None;
                self.prompt = None;
                match crate::auth::save_tokens(&tokens) {
                    Ok(()) => {
                        self.picking_agent = false;
                        self.in_setup = false;
                        self.status_msg = "BrainPal connected — jobs talk to Grok".into();
                    }
                    Err(e) => self.status_msg = format!("save token failed: {e}"),
                }
            }
            Ok(Err(e)) => {
                self.oauth_rx = None;
                self.status_msg = format!("login failed: {e}");
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.oauth_rx = None;
            }
        }
    }

    pub fn run_buddy_job(&mut self) {
        let job = self
            .buddy_items
            .get(self.buddy_index)
            .cloned()
            .unwrap_or_default();
        let body = self.textarea.lines().join("\n");

        if job == "Setup BrainPal" {
            if crate::ai::has_auth() {
                match crate::auth::logout() {
                    Ok(()) => {
                        self.picking_agent = true;
                        self.in_setup = true;
                        self.status_msg = "BrainPal disconnected".into();
                    }
                    Err(e) => self.status_msg = format!("disconnect failed: {e}"),
                }
            } else {
                self.start_selected_agent();
            }
            return;
        }

        // Prefer OAuth Grok when logged in; else local stub.
        let online = crate::ai::has_auth();
        self.status_msg = if online {
            format!("calling Grok… ({job})")
        } else {
            format!("local stub ({job}) — Setup BrainPal to connect")
        };

        match job.as_str() {
            "Unstuck" | "Another" | "Another take" => {
                let text = if online {
                    match crate::ai::unstuck(&body, None) {
                        Ok(t) => t,
                        Err(e) => {
                            self.status_msg = format!("Grok failed, local: {e}");
                            local_ai::unstuck_paragraph(&body)
                        }
                    }
                } else {
                    local_ai::unstuck_paragraph(&body)
                };
                let text = if job.starts_with("Another") {
                    format!("(another)\n{text}")
                } else {
                    text
                };
                self.insert_proposal(text);
            }
            "Verify" | "Verify names" => {
                let names = self.library.character_names();
                let text = if online {
                    match crate::ai::verify(&body, &names) {
                        Ok(t) => t,
                        Err(e) => {
                            self.status_msg = format!("Grok failed, local: {e}");
                            local_ai::verify_names(&body, &names)
                        }
                    }
                } else {
                    local_ai::verify_names(&body, &names)
                };
                self.insert_proposal(text);
            }
            "Refine" | "Refine note" => {
                let text = if online {
                    match crate::ai::refine(&body) {
                        Ok(t) => t,
                        Err(e) => {
                            self.status_msg = format!("Grok failed, local: {e}");
                            local_ai::refine_note(&body)
                        }
                    }
                } else {
                    local_ai::refine_note(&body)
                };
                if text.is_empty() {
                    self.status_msg = "nothing to refine".into();
                } else {
                    self.insert_proposal(format!("Refined:\n{text}"));
                }
            }
            "Gist" => {
                match self.library.write_gist_active() {
                    Ok(()) => self.status_msg = "gist written — Outline will show it".into(),
                    Err(e) => self.status_msg = format!("gist failed: {e}"),
                }
            }
            "Ask" => {
                let q = if body.len() > 800 {
                    format!("Answer briefly about this page:\n{}", &body[..800])
                } else {
                    format!("Answer briefly about this page:\n{body}")
                };
                let text = if online {
                    match crate::ai::complete(
                        "You are BrainPal in Elfy. Short, concrete answers. No fluff.",
                        &q,
                    ) {
                        Ok(t) => t,
                        Err(e) => format!("(offline) {e}"),
                    }
                } else {
                    "Setup BrainPal first — press S in this pane, pick Grok, Enter.".into()
                };
                self.insert_proposal(text);
            }
            "Continuity" => {
                let flags_local = self.library.local_continuity_scan();
                let text = if online {
                    let truth: String = self
                        .library
                        .list_docs(crate::library::Folder::Characters)
                        .into_iter()
                        .map(|d| format!("## {}\n{}", d.name, d.body))
                        .collect::<Vec<_>>()
                        .join("\n\n");
                    match crate::ai::continuity(&body, &truth) {
                        Ok(t) => t,
                        Err(e) => {
                            self.status_msg = format!("Grok failed, local: {e}");
                            flags_local.join("\n")
                        }
                    }
                } else {
                    flags_local.join("\n")
                };
                self.insert_proposal(text);
            }
            _ => {
                self.status_msg = format!("{job} — not wired yet").into();
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.prompt.is_some() {
            self.handle_prompt(key);
            return;
        }

        // global
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }
        if key.code == KeyCode::F(8) {
            self.cycle_theme();
            return;
        }
        if key.code == KeyCode::F(1) {
            self.help_visible = !self.help_visible;
            return;
        }
        if key.code == KeyCode::F(10) {
            self.focus = Focus::Menu;
            return;
        }
        if key.code == KeyCode::F(6)
            || key.code == KeyCode::F(7)
            || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('b'))
        {
            self.focus_agent_pane();
            return;
        }

        // proposal keys take priority in workspace
        if self.proposal.is_some() && self.focus == Focus::Workspace {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.keep_proposal();
                    return;
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.toss_proposal();
                    return;
                }
                KeyCode::Esc => {
                    self.toss_proposal();
                    return;
                }
                KeyCode::Tab => {
                    // cycle to next buddy job and re-run
                    self.buddy_index = (self.buddy_index + 1) % self.buddy_items.len();
                    self.toss_proposal();
                    self.run_buddy_job();
                    return;
                }
                _ => {}
            }
        }

        // Esc double-tap → menu
        if key.code == KeyCode::Esc {
            let now = std::time::Instant::now();
            if let Some(t) = self.last_esc {
                if now.duration_since(t).as_millis() < 400 {
                    self.focus = Focus::Menu;
                    self.last_esc = None;
                    return;
                }
            }
            self.last_esc = Some(now);
            if self.focus == Focus::Buddy {
                self.focus = Focus::Workspace;
            } else if self.editing_path.is_some() {
                // back to list
                self.editing_path = None;
                self.dirty = false;
            } else {
                self.focus = Focus::Menu;
            }
            return;
        }

        match self.focus {
            Focus::Menu => self.handle_menu_key(key),
            Focus::Buddy => self.handle_buddy_key(key),
            Focus::Workspace => self.handle_workspace_key(key),
        }
    }

    fn handle_menu_key(&mut self, key: KeyEvent) {
        let vis = self.visible_menus();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.menu_index > 0 {
                    self.menu_index -= 1;
                } else {
                    self.menu_index = vis.len().saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.menu_index = (self.menu_index + 1) % vis.len().max(1);
            }
            KeyCode::Enter => {
                if let Some(id) = vis.get(self.menu_index) {
                    self.enter_room(*id);
                }
            }
            KeyCode::Char(c) => {
                if let Some(id) = MenuId::from_letter(c) {
                    if let Some(pos) = vis.iter().position(|m| *m == id) {
                        self.menu_index = pos;
                        self.enter_room(id);
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_buddy_key(&mut self, key: KeyEvent) {
        let connected = crate::ai::has_auth();
        match key.code {
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.in_setup = true;
                self.picking_agent = !connected;
                self.status_msg = "Setup BrainPal — pick agent, Enter to connect".into();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !connected {
                    let n = crate::auth::AgentKind::ALL.len();
                    self.agent_index = (self.agent_index + n - 1) % n;
                } else if self.buddy_index > 0 {
                    self.buddy_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !connected {
                    let n = crate::auth::AgentKind::ALL.len();
                    self.agent_index = (self.agent_index + 1) % n;
                } else if self.buddy_index + 1 < self.buddy_items.len() {
                    self.buddy_index += 1;
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                let _ = crate::auth::logout();
                self.picking_agent = true;
                self.in_setup = true;
                self.status_msg = "BrainPal disconnected".into();
            }
            KeyCode::Enter => {
                if !connected {
                    self.start_selected_agent();
                } else {
                    self.run_buddy_job();
                }
            }
            _ => {}
        }
    }

    fn wrap_rows(&self) -> Vec<wrap::VRow> {
        let lines: Vec<String> = self.textarea.lines().iter().map(|s| s.to_string()).collect();
        wrap::wrap_doc(&lines, self.wrap_width.max(12))
    }

    fn jump_cursor(&mut self, line: usize, col: usize) {
        let line = line.min(u16::MAX as usize) as u16;
        let col = col.min(u16::MAX as usize) as u16;
        self.textarea.move_cursor(CursorMove::Jump(line, col));
    }

    fn move_visual(&mut self, drow: i32) {
        let rows = self.wrap_rows();
        if rows.is_empty() {
            return;
        }
        let (line, col) = self.textarea.cursor();
        let (vrow, _) = wrap::visual_pos(&rows, line, col);
        let dest = if drow < 0 {
            vrow.saturating_sub((-drow) as usize)
        } else {
            (vrow + drow as usize).min(rows.len().saturating_sub(1))
        };
        let (nl, nc) = wrap::logical_pos(&rows, dest, self.vis_col);
        self.jump_cursor(nl, nc);
    }

    fn move_visual_home(&mut self) {
        let rows = self.wrap_rows();
        let (line, col) = self.textarea.cursor();
        let (vrow, _) = wrap::visual_pos(&rows, line, col);
        let (nl, nc) = wrap::logical_pos(&rows, vrow, 0);
        self.vis_col = 0;
        self.jump_cursor(nl, nc);
    }

    fn move_visual_end(&mut self) {
        let rows = self.wrap_rows();
        let (line, col) = self.textarea.cursor();
        let (vrow, _) = wrap::visual_pos(&rows, line, col);
        let width = rows.get(vrow).map(|r| r.end.saturating_sub(r.start)).unwrap_or(0);
        let (nl, nc) = wrap::logical_pos(&rows, vrow, width);
        self.vis_col = width;
        self.jump_cursor(nl, nc);
    }

    fn remember_vis_col(&mut self) {
        let rows = self.wrap_rows();
        let (line, col) = self.textarea.cursor();
        let (_, vcol) = wrap::visual_pos(&rows, line, col);
        self.vis_col = vcol;
    }

    fn handle_editor_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.move_visual(-1),
            KeyCode::Down => self.move_visual(1),
            KeyCode::PageUp => {
                let h = self.write_height.max(1) as i32;
                self.move_visual(-h);
            }
            KeyCode::PageDown => {
                let h = self.write_height.max(1) as i32;
                self.move_visual(h);
            }
            KeyCode::Home => self.move_visual_home(),
            KeyCode::End => self.move_visual_end(),
            _ => {
                self.textarea.input(key_to_input(key));
                self.dirty = true;
                self.remember_vis_col();
            }
        }
    }

    fn handle_workspace_key(&mut self, key: KeyEvent) {
        // F-keys
        if key.code == KeyCode::F(2) {
            self.save();
            return;
        }
        if key.code == KeyCode::F(3) && self.menu_id == MenuId::Write {
            self.chapter_picker = !self.chapter_picker;
            if self.chapter_picker {
                self.list_index = self.library.active_chapter;
            }
            return;
        }

        // chapter picker overlay
        if self.chapter_picker {
            let n = self
                .library
                .book()
                .map(|b| b.chapters.len())
                .unwrap_or(0);
            match key.code {
                KeyCode::Esc => {
                    self.chapter_picker = false;
                }
                KeyCode::Enter => {
                    if self.list_index < n {
                        self.library.active_chapter = self.list_index;
                        self.chapter_picker = false;
                        self.load_active_into_editor();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.list_index > 0 {
                        self.list_index -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.list_index + 1 < n {
                        self.list_index += 1;
                    }
                }
                _ => {}
            }
            return;
        }

        match self.menu_id {
            MenuId::Books => match key.code {
                KeyCode::Char('n') | KeyCode::Char('N') => self.start_new_book(),
                KeyCode::Char('e') | KeyCode::Char('E') => self.start_rename_book(),
                KeyCode::Char('d') | KeyCode::Char('D') => self.start_delete_book(),
                KeyCode::Enter => {
                    let idx = self.list_index;
                    if idx < self.library.books.len() {
                        self.library.set_active_book(idx);
                        self.enter_room(MenuId::Write);
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.list_index > 0 {
                        self.list_index -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.list_index + 1 < self.library.books.len() {
                        self.list_index += 1;
                    }
                }
                _ => {}
            },
            MenuId::Write => {
                if key.code == KeyCode::Tab {
                    self.library.next_chapter();
                    self.load_active_into_editor();
                    return;
                }
                self.handle_editor_key(key);
            }
            MenuId::Ideas | MenuId::Characters | MenuId::Lore | MenuId::Style => {
                if self.editing_path.is_some() {
                    if key.code == KeyCode::F(2) {
                        self.save();
                        return;
                    }
                    self.handle_editor_key(key);
                } else {
                    let folder = self.menu_id.folder().unwrap();
                    let docs = self.library.list_docs(folder);
                    match key.code {
                        KeyCode::Char('n') | KeyCode::Char('N') => {
                            self.start_new_doc(folder);
                        }
                        KeyCode::Enter => {
                            if let Some(doc) = docs.get(self.list_index) {
                                self.load_path_into_editor(doc.path.clone(), doc.body.clone());
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if self.list_index > 0 {
                                self.list_index -= 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if self.list_index + 1 < docs.len() {
                                self.list_index += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
            MenuId::Adventures => {
                if self.editing_path.is_some() {
                    if key.code == KeyCode::F(2) {
                        self.save();
                        return;
                    }
                    self.handle_editor_key(key);
                } else {
                    let docs = self.library.list_adventures();
                    match key.code {
                        KeyCode::Enter => {
                            if let Some(doc) = docs.get(self.list_index) {
                                self.load_path_into_editor(doc.path.clone(), doc.body.clone());
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if self.list_index > 0 {
                                self.list_index -= 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if self.list_index + 1 < docs.len() {
                                self.list_index += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
            MenuId::Review => {
                let items = self.library.list_review();
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if self.review_index > 0 {
                            self.review_index -= 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if self.review_index + 1 < items.len() {
                            self.review_index += 1;
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(item) = items.get(self.review_index) {
                            let mut ta =
                                TextArea::from(item.body.lines().map(|l| l.to_string()));
                            ta.set_cursor_style(Style::default());
                            self.textarea = ta;
                            self.style_editor();
                            self.status_msg = format!("review: {}", item.title);
                        }
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        if let Some(item) = items.get(self.review_index).cloned() {
                            match self.library.resolve_review(&item, true) {
                                Ok(()) => {
                                    let _ = self.library.git_commit(&format!(
                                        "review approve {}",
                                        item.id
                                    ));
                                    self.status_msg = format!("approved {}", item.title);
                                    if self.review_index > 0 && self.review_index >= items.len().saturating_sub(1) {
                                        self.review_index -= 1;
                                    }
                                }
                                Err(e) => self.status_msg = format!("review failed: {e}"),
                            }
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Char('s')
                    | KeyCode::Char('S') => {
                        if let Some(item) = items.get(self.review_index).cloned() {
                            match self.library.resolve_review(&item, false) {
                                Ok(()) => {
                                    let _ = self
                                        .library
                                        .git_commit(&format!("review skip {}", item.id));
                                    self.status_msg = format!("skipped {}", item.title);
                                    if self.review_index > 0
                                        && self.review_index >= items.len().saturating_sub(1)
                                    {
                                        self.review_index -= 1;
                                    }
                                }
                                Err(e) => self.status_msg = format!("review failed: {e}"),
                            }
                        }
                    }
                    _ => {}
                }
            }
            MenuId::Outline | MenuId::Timeline => {
                // read-only text surfaces
            }
            _ => {}
        }
    }
}

fn default_buddy() -> Vec<String> {
    vec![
        "Unstuck".into(),
        "Verify".into(),
        "Refine".into(),
        "Another".into(),
        "Gist".into(),
        "Ask".into(),
        "Continuity".into(),
    ]
}

fn key_to_input(ev: KeyEvent) -> Input {
    let k = match ev.code {
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter => Key::Enter,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Delete => Key::Delete,
        KeyCode::Tab => Key::Tab,
        KeyCode::Esc => Key::Null,
        _ => Key::Null,
    };
    Input {
        key: k,
        ctrl: ev.modifiers.contains(KeyModifiers::CONTROL),
        alt: ev.modifiers.contains(KeyModifiers::ALT),
        shift: ev.modifiers.contains(KeyModifiers::SHIFT),
    }
}
