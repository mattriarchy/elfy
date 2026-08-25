//! Local (no-API) helpers for Buddy jobs.
//! Real prose AI still needs a key. These give Unstuck/Verify/Refine something honest to do.

/// Next-paragraph draft from the end of the chapter. Accent-colored proposal.
pub fn unstuck_paragraph(body: &str) -> String {
    let paras: Vec<&str> = body
        .split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty() && !p.starts_with('#'))
        .collect();
    let last = paras.last().copied().unwrap_or("");
    if last.is_empty() {
        return "The room waited. He had not decided what to do next.".into();
    }

    // last sentence
    let last_sent = last
        .rsplit_once('.')
        .map(|(a, _)| format!("{}.", a.trim()))
        .unwrap_or_else(|| last.to_string());
    let seed = last_sent.trim();

    // Extract a capitalized name if present
    let mut name = String::new();
    for w in seed.split_whitespace() {
        let clean: String = w.chars().filter(|c| c.is_alphabetic()).collect();
        if clean.len() > 2 && clean.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            name = clean;
            break;
        }
    }

    if name.is_empty() {
        "A beat passed. What came next was smaller than the last moment — and harder to take back.\n\nHe moved before the choice could finish forming.".into()
    } else {
        format!(
            "{name} did not look away. The next move was already in the room with them, waiting to be named.\n\nWhen it came, it was quieter than anyone expected."
        )
    }
}

/// Scan chapter for character names that exist in Truth files.
pub fn verify_names(body: &str, names: &[String]) -> String {
    if names.is_empty() {
        return "No characters on file. Add one under Characters.".into();
    }
    let lower = body.to_ascii_lowercase();
    let mut present = Vec::new();
    let mut missing = Vec::new();
    for n in names {
        if lower.contains(&n.to_ascii_lowercase()) {
            present.push(n.as_str());
        } else {
            missing.push(n.as_str());
        }
    }
    let mut out = String::new();
    if !present.is_empty() {
        out.push_str(&format!("On page: {}.\n", present.join(", ")));
    }
    if !missing.is_empty() {
        out.push_str(&format!(
            "In Truth, not on this page: {}.\n",
            missing.join(", ")
        ));
    }
    if out.is_empty() {
        out.push_str("Nothing to compare.\n");
    }
    out
}

/// Collapse a raw note into a single clean intention statement.
pub fn refine_note(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        return String::new();
    }
    let one: String = t.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.ends_with('.') || one.ends_with('!') || one.ends_with('?') {
        one
    } else {
        format!("{one}.")
    }
}
