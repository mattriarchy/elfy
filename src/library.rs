use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Chapter {
    pub path: PathBuf,
    pub filename: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct SideDoc {
    pub path: PathBuf,
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewKind {
    Continuity,
    Outline,
    Refiner,
    Other,
}

#[derive(Debug, Clone)]
pub struct ReviewItem {
    pub id: String,
    pub kind: ReviewKind,
    pub title: String,
    pub body: String,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Folder {
    Characters,
    Ideas,
    Lore,
    Style,
}

impl Folder {
    pub fn dir_name(self) -> &'static str {
        match self {
            Folder::Characters => "characters",
            Folder::Ideas => "inbox",
            Folder::Lore => "lore",
            Folder::Style => "style",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Book {
    pub slug: String,
    pub title: String,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone)]
pub struct Library {
    pub root: PathBuf,
    pub books: Vec<Book>,
    pub active_book: usize,
    pub active_chapter: usize,
}

impl Library {
    pub fn open_default() -> Result<Self> {
        let root = resolve_library_root();
        fs::create_dir_all(root.join("books"))
            .with_context(|| format!("creating {}", root.display()))?;
        maybe_seed(&root)?;
        Self::load(root)
    }

    fn load(root: PathBuf) -> Result<Self> {
        let books_dir = root.join("books");
        let mut books = Vec::new();

        if books_dir.is_dir() {
            let mut entries: Vec<_> = fs::read_dir(&books_dir)
                .with_context(|| format!("reading {}", books_dir.display()))?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .collect();
            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let slug = entry.file_name().to_string_lossy().to_string();
                let book_path = entry.path();
                let title = read_config(&book_path);
                let chapters = load_chapters(&book_path);
                books.push(Book {
                    slug,
                    title,
                    chapters,
                });
            }
        }

        let active_book = 0;
        let active_chapter = 0;
        Ok(Self {
            root,
            books,
            active_book,
            active_chapter,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.books.is_empty()
    }

    pub fn book(&self) -> Option<&Book> {
        self.books.get(self.active_book)
    }

    pub fn book_mut(&mut self) -> Option<&mut Book> {
        self.books.get_mut(self.active_book)
    }

    pub fn chapter(&self) -> Option<&Chapter> {
        self.book()
            .and_then(|b| b.chapters.get(self.active_chapter))
    }

    pub fn set_chapter_body(&mut self, body: String) {
        let i = self.active_chapter;
        if let Some(book) = self.book_mut() {
            if let Some(ch) = book.chapters.get_mut(i) {
                ch.body = body;
            }
        }
    }

    pub fn save_active(&self) -> Result<()> {
        let ch = self.chapter().context("no active chapter")?;
        fs::write(&ch.path, &ch.body)
            .with_context(|| format!("writing {}", ch.path.display()))?;
        Ok(())
    }

    pub fn write_gist_active(&self) -> Result<()> {
        let ch = self.chapter().context("no active chapter")?;
        let gist = naive_gist(&ch.body);
        let gist_path = ch.path.with_extension("gist.md");
        fs::write(&gist_path, gist)
            .with_context(|| format!("writing gist {}", gist_path.display()))?;
        Ok(())
    }

    pub fn next_chapter(&mut self) {
        if let Some(book) = self.book() {
            if !book.chapters.is_empty() {
                self.active_chapter = (self.active_chapter + 1) % book.chapters.len();
            }
        }
    }

    pub fn prev_chapter(&mut self) {
        if let Some(book) = self.book() {
            if !book.chapters.is_empty() {
                let n = book.chapters.len();
                self.active_chapter = (self.active_chapter + n - 1) % n;
            }
        }
    }

    pub fn set_active_book(&mut self, idx: usize) {
        if idx < self.books.len() {
            self.active_book = idx;
            self.active_chapter = 0;
        }
    }

    pub fn create_book(&mut self, title: &str) -> Result<()> {
        let slug = slugify(title);
        let book_path = self.root.join("books").join(&slug);
        if book_path.exists() {
            anyhow::bail!("book already exists: {slug}");
        }
        ensure_layout(&book_path)?;
        let config = format!("title: \"{}\"\n", title);
        fs::write(book_path.join("config.yaml"), config)?;
        let ch_name = "ch-01.md";
        let body = format!("{}\n\n", elf_permission(title));
        fs::write(book_path.join("manuscript").join(ch_name), &body)?;

        self.books.push(Book {
            slug,
            title: title.to_string(),
            chapters: vec![Chapter {
                path: book_path.join("manuscript").join(ch_name),
                filename: ch_name.into(),
                title: title.to_string(),
                body,
            }],
        });
        self.active_book = self.books.len() - 1;
        self.active_chapter = 0;
        Ok(())
    }

    /// Change the display title. Slug stays put so git history and paths do not break.
    pub fn rename_book(&mut self, index: usize, new_title: &str) -> Result<()> {
        let title = new_title.trim();
        if title.is_empty() {
            anyhow::bail!("empty title");
        }
        if index >= self.books.len() {
            anyhow::bail!("bad index");
        }
        let slug = self.books[index].slug.clone();
        let cfg = self.root.join("books").join(&slug).join("config.yaml");
        fs::write(&cfg, format!("title: \"{}\"\n", title))?;
        self.books[index].title = title.to_string();
        Ok(())
    }

    pub fn delete_book(&mut self, index: usize) -> Result<()> {
        if index >= self.books.len() {
            anyhow::bail!("bad index");
        }
        let slug = self.books[index].slug.clone();
        let path = self.root.join("books").join(&slug);
        if path.exists() {
            fs::remove_dir_all(&path)
                .with_context(|| format!("removing {}", path.display()))?;
        }
        self.books.remove(index);
        if self.active_book >= self.books.len() && !self.books.is_empty() {
            self.active_book = self.books.len() - 1;
        }
        if self.books.is_empty() {
            self.active_book = 0;
        }
        self.active_chapter = 0;
        Ok(())
    }

    pub fn list_docs(&self, folder: Folder) -> Vec<SideDoc> {
        let Some(book) = self.book() else {
            return Vec::new();
        };
        let dir = self
            .root
            .join("books")
            .join(&book.slug)
            .join(folder.dir_name());
        if !dir.is_dir() {
            return Vec::new();
        }
        let mut docs = Vec::new();
        if let Ok(entries) = fs::read_dir(&dir) {
            let mut paths: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|x| x == "md")
                        .unwrap_or(false)
                })
                .collect();
            paths.sort_by_key(|e| e.file_name());
            for e in paths {
                let path = e.path();
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let body = fs::read_to_string(&path).unwrap_or_default();
                docs.push(SideDoc { path, name, body });
            }
        }
        docs
    }

    pub fn create_doc(&self, folder: Folder, name: &str) -> Result<PathBuf> {
        let book = self.book().context("no book")?;
        let dir = self
            .root
            .join("books")
            .join(&book.slug)
            .join(folder.dir_name());
        fs::create_dir_all(&dir)?;
        let filename = format!("{}.md", slugify(name));
        let path = dir.join(&filename);
        if path.exists() {
            anyhow::bail!("already exists");
        }
        let body = match folder {
            Folder::Characters => format!(
                "# {}\n\n## Truth\n\n- \n\n## Observed\n\n- \n",
                name
            ),
            Folder::Ideas => format!("# {}\n\n", name),
            Folder::Lore => format!("# {}\n\n", name),
            Folder::Style => format!("# {}\n\n", name),
        };
        fs::write(&path, body)?;
        Ok(path)
    }

    pub fn save_path(&self, path: &Path, body: &str) -> Result<()> {
        fs::write(path, body).with_context(|| format!("writing {}", path.display()))?;
        Ok(())
    }

    pub fn character_names(&self) -> Vec<String> {
        self.list_docs(Folder::Characters)
            .into_iter()
            .map(|d| d.name)
            .collect()
    }

    pub fn outline_text(&self) -> String {
        let Some(book) = self.book() else {
            return String::new();
        };
        let mut out = String::new();
        out.push_str(&format!("# Outline — {}\n\n", book.title));
        for (i, ch) in book.chapters.iter().enumerate() {
            let marker = if i == self.active_chapter { "▸" } else { " " };
            out.push_str(&format!("{} {}. {}\n", marker, i + 1, ch.title));
            let gist_path = ch.path.with_extension("gist.md");
            if let Ok(g) = fs::read_to_string(&gist_path) {
                for line in g.lines().take(6) {
                    if !line.trim().is_empty() {
                        out.push_str(&format!("     {}\n", line.trim()));
                    }
                }
            }
            out.push('\n');
        }
        out
    }

    /// Load pending review items from books/<slug>/review/*.md
    pub fn list_review(&self) -> Vec<ReviewItem> {
        let Some(book) = self.book() else {
            return Vec::new();
        };
        let dir = self.root.join("books").join(&book.slug).join("review");
        if !dir.is_dir() {
            return Vec::new();
        }
        let mut items = Vec::new();
        if let Ok(entries) = fs::read_dir(&dir) {
            let mut paths: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
                .collect();
            paths.sort_by_key(|e| e.file_name());
            for e in paths {
                let path = e.path();
                let body = fs::read_to_string(&path).unwrap_or_default();
                let title = body
                    .lines()
                    .find(|l| l.starts_with('#'))
                    .map(|l| l.trim_start_matches('#').trim().to_string())
                    .unwrap_or_else(|| {
                        path.file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default()
                    });
                let kind = if title.to_ascii_lowercase().contains("outline") {
                    ReviewKind::Outline
                } else if title.to_ascii_lowercase().contains("contin") {
                    ReviewKind::Continuity
                } else if title.to_ascii_lowercase().contains("refin") {
                    ReviewKind::Refiner
                } else {
                    ReviewKind::Other
                };
                let id = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                items.push(ReviewItem {
                    id,
                    kind,
                    title,
                    body,
                    path: Some(path),
                });
            }
        }
        items
    }

    pub fn resolve_review(&self, item: &ReviewItem, approve: bool) -> Result<()> {
        let Some(path) = &item.path else {
            return Ok(());
        };
        let book = self.book().context("no book")?;
        let archive = self
            .root
            .join("books")
            .join(&book.slug)
            .join("review")
            .join("archive");
        fs::create_dir_all(&archive)?;
        let dest_name = format!(
            "{}-{}.md",
            if approve { "ok" } else { "skip" },
            item.id
        );
        let dest = archive.join(dest_name);
        fs::rename(path, &dest)?;
        Ok(())
    }

    pub fn list_adventures(&self) -> Vec<SideDoc> {
        let Some(book) = self.book() else {
            return Vec::new();
        };
        let dir = self
            .root
            .join("books")
            .join(&book.slug)
            .join("rpg")
            .join("adventures");
        if !dir.is_dir() {
            return Vec::new();
        }
        let mut docs = Vec::new();
        if let Ok(entries) = fs::read_dir(&dir) {
            let mut paths: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
                .collect();
            paths.sort_by_key(|e| e.file_name());
            for e in paths {
                let path = e.path();
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let body = fs::read_to_string(&path).unwrap_or_default();
                docs.push(SideDoc { path, name, body });
            }
        }
        docs
    }

    /// Local continuity scan: names in Truth but never appearing in any chapter.
    pub fn local_continuity_scan(&self) -> Vec<String> {
        let names = self.character_names();
        if names.is_empty() {
            return vec!["No characters on file.".into()];
        }
        let mut flags = Vec::new();
        let mut all_text = String::new();
        if let Some(book) = self.book() {
            for ch in &book.chapters {
                all_text.push_str(&ch.body);
                all_text.push('\n');
            }
        }
        let lower = all_text.to_ascii_lowercase();
        for n in &names {
            if !lower.contains(&n.to_ascii_lowercase()) {
                flags.push(format!(
                    "Truth character «{n}» never appears in manuscript."
                ));
            }
        }
        // simple letter check for 1771 sample
        if lower.contains("letter") && !lower.contains("table") {
            flags.push(
                "Chapter mentions a letter but not the table it was left on (Truth says table)."
                    .into(),
            );
        }
        if flags.is_empty() {
            flags.push("No local continuity flags.".into());
        }
        flags
    }

    pub fn timeline_text(&self) -> String {
        let mut out = String::from("# Timeline (local scan)\n\n");
        out.push_str("Events inferred from chapter order + gists. Real extraction is v3.\n\n");
        if let Some(book) = self.book() {
            for (i, ch) in book.chapters.iter().enumerate() {
                out.push_str(&format!("{}. **{}**\n", i + 1, ch.title));
                let gist_path = ch.path.with_extension("gist.md");
                if let Ok(g) = fs::read_to_string(&gist_path) {
                    for line in g.lines().take(4) {
                        if !line.trim().is_empty() {
                            out.push_str(&format!("   {}\n", line.trim()));
                        }
                    }
                } else {
                    // first non-empty sentence
                    for para in ch.body.split("\n\n") {
                        let t = para.trim();
                        if !t.is_empty() && !t.starts_with('#') {
                            let sent: String = t
                                .split_whitespace()
                                .take(14)
                                .collect::<Vec<_>>()
                                .join(" ");
                            out.push_str(&format!("   - {sent}…\n"));
                            break;
                        }
                    }
                }
                out.push('\n');
            }
        }
        let flags = self.local_continuity_scan();
        out.push_str("## Flags\n\n");
        for f in flags {
            out.push_str(&format!("- {f}\n"));
        }
        out
    }

    /// Ensure the library root is a git repo and commit the given paths.
    pub fn git_commit(&self, message: &str) -> Result<()> {
        use std::process::Command;
        let root = &self.root;
        // init if needed
        if !root.join(".git").exists() {
            let st = Command::new("git")
                .args(["init"])
                .current_dir(root)
                .output()?;
            if !st.status.success() {
                anyhow::bail!("git init failed");
            }
        }
        let _ = Command::new("git")
            .args(["add", "-A"])
            .current_dir(root)
            .output();
        let st = Command::new("git")
            .args(["commit", "-m", message, "--allow-empty"])
            .current_dir(root)
            .output()?;
        if !st.status.success() {
            // nothing to commit is fine
            let stderr = String::from_utf8_lossy(&st.stderr);
            if !stderr.contains("nothing to commit") && !stderr.is_empty() {
                // soft-fail; don't block save
            }
        }
        Ok(())
    }
}

fn resolve_library_root() -> PathBuf {
    if let Ok(p) = std::env::var("ELFY_LIBRARY") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("writing").join("elfy-library")
}

fn read_config(book_path: &Path) -> String {
    let cfg = book_path.join("config.yaml");
    let raw = fs::read_to_string(&cfg).unwrap_or_default();
    let mut title = book_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".into());
    for line in raw.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("title:") {
            title = rest
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
        }
    }
    title
}

fn load_chapters(book_path: &Path) -> Vec<Chapter> {
    let ms = book_path.join("manuscript");
    if !ms.is_dir() {
        return Vec::new();
    }
    let mut chapters = Vec::new();
    if let Ok(entries) = fs::read_dir(&ms) {
        let mut paths: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let p = e.path();
                p.extension().map(|x| x == "md").unwrap_or(false)
                    && !p
                        .file_name()
                        .map(|n| n.to_string_lossy().contains(".gist."))
                        .unwrap_or(false)
            })
            .collect();
        paths.sort_by_key(|e| e.file_name());
        for e in paths {
            let path = e.path();
            let filename = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let body = fs::read_to_string(&path).unwrap_or_default();
            let title = body
                .lines()
                .find(|l| l.starts_with('#'))
                .map(|l| l.trim_start_matches('#').trim().to_string())
                .unwrap_or_else(|| filename.clone());
            chapters.push(Chapter {
                path,
                filename,
                title,
                body,
            });
        }
    }
    chapters
}

pub fn ensure_layout(book_path: &Path) -> Result<()> {
    let dirs = [
        "manuscript",
        "inbox",
        "inbox/archive",
        "characters",
        "lore",
        "outline",
        "style",
        "understanding",
        "understanding/intentions",
        "annotations",
        "review",
    ];
    for d in dirs {
        fs::create_dir_all(book_path.join(d))?;
    }
    Ok(())
}

pub fn elf_permission(title: &str) -> String {
    format!(
        "# {title}\n\n> This manuscript was produced by an incompetent hired elf.\n> It is complete and terrible on purpose. Do not polish it.\n> Write the second draft from scratch."
    )
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if c == ' ' || c == '-' || c == '_' {
            if !out.ends_with('-') {
                out.push('-');
            }
        }
    }
    out.trim_matches('-').to_string()
}

fn naive_gist(body: &str) -> String {
    let mut bullets = Vec::new();
    for para in body.split("\n\n") {
        let t = para.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let first: String = t
            .split_whitespace()
            .take(12)
            .collect::<Vec<_>>()
            .join(" ");
        if first.len() > 8 {
            bullets.push(format!("- {first}…"));
        }
        if bullets.len() >= 5 {
            break;
        }
    }
    if bullets.is_empty() {
        "- (empty)\n".into()
    } else {
        bullets.join("\n") + "\n"
    }
}

fn maybe_seed(root: &Path) -> Result<()> {
    let books = root.join("books");
    if books.read_dir()?.next().is_some() {
        return Ok(());
    }
    let path = books.join("the-hired-elf");
    ensure_layout(&path)?;
    fs::write(path.join("config.yaml"), "title: \"The Hired Elf\"\n")?;
    let ch1 = r#"# The Hired Elf

> This manuscript was produced by an incompetent hired elf.
> It is complete and terrible on purpose. Do not polish it.
> Write the second draft from scratch.

Okay so there is this guy named Whitcomb or maybe Whitmore I forget and he walks out of a town that is muddy. The mud is very muddy. He left a letter on a table which is important probably.

He is wearing a coat. The coat is wet. Trees are there. Light is pewter which I think is a metal. Anyway he keeps walking because that is what walking people do.

Somebody at a meeting house later says you left something. He nods. The room waits. The elf who wrote this does not know what happens after the nod so the room just keeps waiting. The end of the chapter is the waiting.
"#;
    let ch2 = r#"# The Meeting House, Badly

They meet under a gallery. The light is hard and white like a lightbulb that is also a church. Whitcomb does not sit. Sitting is for people who sit.

"You left something," said a woman with a ledger, which is a book of numbers I guess.
He nodded once. The room waited some more.

Then there is soup. Why is there soup. The elf put soup in because chapters need food. The soup is too hot. Nobody eats it. Continuity is already in trouble.
"#;
    fs::write(path.join("manuscript/ch-01.md"), ch1)?;
    fs::write(path.join("manuscript/ch-02.md"), ch2)?;
    fs::write(
        path.join("characters/whitcomb.md"),
        "# Whitcomb\n\n## Truth\n\n- Left a letter on a table\n- Walks in weather\n\n## Observed\n\n- Does not sit\n- Nods once\n",
    )?;
    fs::write(
        path.join("inbox/note-letter.md"),
        "# The letter\n\nHe left it. The elf does not know what it says. That is allowed.\n",
    )?;
    fs::write(
        path.join("lore/the-town.md"),
        "# The town\n\nMud. A meeting house. Soup that should not be there.\n",
    )?;
    fs::write(
        path.join("style/voice.md"),
        "# Voice\n\nElf draft: complete, wrong, fast. Do not sand the sentences.\n",
    )?;
    fs::write(
        path.join("review/contin-soup.md"),
        "# Continuity — soup\n\nChapter 2 invents soup. Nothing in Truth or Lore mentions soup.\n\nY keep the soup (elf draft is allowed to be wrong).\nN/S skip the flag.\n",
    )?;
    Ok(())
}
