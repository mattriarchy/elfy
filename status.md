# Elfy — Status

Copied from `mattriarchy/book-processor`, then stripped.

## 2026-08-24 — Fork from Book Processor

- Product: **Elfy** (`elfy` binary). Library `~/writing/elfy-library` / `$ELFY_LIBRARY`.
- Config tokens: `~/.config/elfy/auth.json`.
- **No book types.** `config.yaml` is `title` only. Create no longer asks novel/RPG/screenplay/nonfiction.
- Mechanics / Adventures rooms dropped from the menu (type does not exist).
- **Rename book:** `E` on the Books list. Changes title; slug stays so git paths do not break.
- New books stamped with the hired-elf permission header.
- Sample seed: **The Hired Elf** (terrible on purpose) instead of 1771 RPG.
- BrainPal / OAuth / F-keys / 3-column TUI / git-on-save copied as-is. Right pane jobs are next to rewrite.
- GitHub: `mattriarchy/elfy` · branch `main`

Host install:

```
cd ~/elfy
git pull
cargo install --path . --force
elfy
```
