//! Provider-agnostic completion. Today: xAI Grok via OAuth bearer.
//! Offline fallback stays in local_ai.rs.

use crate::auth::{self, TokenSet};
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

const XAI_CHAT: &str = "https://api.x.ai/v1/chat/completions";
const DEFAULT_MODEL: &str = "grok-4";

pub fn has_auth() -> bool {
    auth::is_logged_in()
}

fn bearer() -> Result<TokenSet> {
    let tokens = auth::load_tokens().context("not logged in — Buddy → Login (OAuth)")?;
    auth::refresh_if_needed(&tokens)
}

/// One-shot chat completion. System + user. Returns assistant text.
pub fn complete(system: &str, user: &str) -> Result<String> {
    let tokens = bearer()?;
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .build()?;

    let body = json!({
        "model": DEFAULT_MODEL,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "temperature": 0.7,
        "max_tokens": 800
    });

    let resp = client
        .post(XAI_CHAT)
        .bearer_auth(&tokens.access_token)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .context("xAI request")?;

    let status = resp.status();
    let text = resp.text().unwrap_or_default();
    if !status.is_success() {
        bail!("xAI {status}: {text}");
    }

    let v: Value = serde_json::from_str(&text).context("parse completion")?;
    let content = v["choices"][0]["message"]["content"]
        .as_str()
        .context("no content in response")?
        .trim()
        .to_string();
    if content.is_empty() {
        bail!("empty completion");
    }
    Ok(content)
}

/// Unstuck: continue the chapter in voice.
pub fn unstuck(chapter_body: &str, style_hint: Option<&str>) -> Result<String> {
    let system = format!(
        "You are a fiction writing partner inside Elfy.\n\
         Continue from the LAST paragraph of the chapter. Stay in the same scene.\n\
         Match the author's voice. Short paragraphs. No headings. No meta commentary.\n\
         Output ONLY the next 1–3 paragraphs of prose.\n\
         {}",
        style_hint.unwrap_or("")
    );
    // send last ~1500 chars as context
    let tail = if chapter_body.len() > 1500 {
        &chapter_body[chapter_body.len() - 1500..]
    } else {
        chapter_body
    };
    let user = format!("Chapter so far (tail):\n\n{tail}\n\nWrite the next beat:");
    complete(&system, &user)
}

/// Refine a raw note into one clean author-intent statement.
pub fn refine(raw: &str) -> Result<String> {
    let system = "You are the Idea Refiner for Elfy.\n\
         Turn the raw note into one clean statement of what the author meant.\n\
         No flourish. No invented plot. Prefer the author's words when precise.\n\
         Output ONLY the refined statement.";
    complete(system, raw)
}

/// Verify: compare page names to character Truth list.
pub fn verify(chapter_body: &str, character_names: &[String]) -> Result<String> {
    let names = if character_names.is_empty() {
        "(none on file)".into()
    } else {
        character_names.join(", ")
    };
    let system = "You are a continuity checker for Elfy.\n\
         Report which character names from Truth appear on this page, and which do not.\n\
         Flag only clear issues. Short bullets. No rewrite suggestions.";
    let user = format!("Truth characters: {names}\n\nPage:\n{chapter_body}");
    complete(&system, &user)
}

/// Continuity flags from chapter + optional truth snippets.
pub fn continuity(chapter_body: &str, truth_blob: &str) -> Result<String> {
    let system = "You are the continuity linter for Elfy.\n\
         Flag contradictions between the page and the Truth/lore notes.\n\
         Each flag: one sentence + the two sources that conflict.\n\
         If none, say so. No rewrites.";
    let user = format!("Truth/lore:\n{truth_blob}\n\nPage:\n{chapter_body}");
    complete(&system, &user)
}
