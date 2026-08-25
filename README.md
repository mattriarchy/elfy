# Elfy

Local-first terminal TUI for **elfing** a book: write a complete, intentionally terrible first draft, then write the second draft from scratch.

Technique (Jon Watts / popularized by Zach Cregger): pretend a hired, incompetent elf produced the manuscript. That permission kills the inner critic. The elf draft is not edited into the book — it is burned and replaced.

WordPerfect lineage. One full-screen surface. Markdown + YAML on disk. Local git safety net. No accounts, no server, no telemetry.

Forked from Book Processor. **No book types.** A book is a book. Rename is first-class (`E` on the Books list).

**Binary:** `elfy`  
**Repo:** `mattriarchy/elfy`  
**Library:** `~/writing/elfy-library` or `$ELFY_LIBRARY`

## Layout

Columns never hide.

```
┌──────────┬──────────────────────────────┬──────────┐
│  MENU    │  WORKSPACE                   │ BrainPal │
│          │                              │          │
│ ▸ Books  │  12 │ The road was mud…      │ SETUP    │
│   Write  │  13 │                        │ Unstuck  │
│   Ideas  │  14 │ ▌                      │ Verify   │
│   Outline│                              │ Refine   │
│   …      │                              │ …        │
├──────────┴──────────────────────────────┴──────────┤
│ the-hired-elf · ch-01.md · 142 words · saved       │
│ F2 save  F3 chapter  F7 BrainPal  F8 theme  F10 menu│
└────────────────────────────────────────────────────┘
```

- **F10** / double-Esc → menu
- **F7** / Ctrl+B → BrainPal
- **F2** → save + gist + git commit
- **F3** → chapter picker
- **F8** → theme (dark / WP-blue / green / amber)
- **F1** → help line
- Books: **N** new · **E** rename (title only; slug stays) · **D** delete (PIN)

## Rooms

| Key | Room |
|-----|------|
| B | Books — list / create / **rename** / delete |
| W | Write — soft-wrap editor + AI proposal chrome |
| I | Ideas |
| O | Outline (gists from F2) |
| C | Characters — Truth / Observed |
| L | Lore |
| S | Style |
| R | Review — Y / N / S + pending count |
| T | Timeline — local scan |

No Mechanics / Adventures. Type does not exist.

New books get an elf-permission header. Do not polish the elf draft. Second draft is a new file, written from scratch.

## BrainPal (right column)

Always visible. Setup is the top block, not a job.

Jobs (will change): Unstuck · Verify · Refine · Another · Gist · Ask · Continuity

Proposal lands in accent color. **Y** keep · **N** toss · **Tab** next.

Grok via device-code OAuth. Tokens: `~/.config/elfy/auth.json` (0600).

## Disk

```
~/writing/elfy-library/
└── books/
    └── the-hired-elf/
        ├── config.yaml          # title only — no type field
        ├── manuscript/          # ch-*.md + .gist.md
        ├── characters/
        ├── inbox/
        ├── lore/
        ├── style/
        ├── outline/
        └── review/
```

`elfy` seeds **The Hired Elf** on first run if the library is empty.

## Build (Linux host)

```bash
cd ~/elfy
cargo build --release
export ELFY_LIBRARY=~/writing/elfy-library   # optional
./target/release/elfy
```

Requires Rust 1.80+.

```bash
cargo install --path . --force
elfy
```

## v1 scope

- [x] TUI shell, 3-column, themes (copied from BP)
- [x] Soft-wrap editor, visual-row arrows
- [x] Books / Write / Ideas / Outline / Characters / Lore / Style / Review / Timeline
- [x] No book types
- [x] Rename book (title; slug stable)
- [x] Elf-permission header on new books
- [x] BrainPal jobs + Grok OAuth slot
- [ ] Right-pane jobs rewritten for elfing (next)
- [ ] Burn-elf → blank second draft command

Right menu and job set are the next cut. Do not hide columns. Do not reintroduce types.
