//! Soft-wrap helpers: logical lines → visual rows, cursor mapping.

#[derive(Debug, Clone)]
pub struct VRow {
    pub logical: usize,
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// Wrap a document into visual rows given a column width.
pub fn wrap_doc(lines: &[String], width: usize) -> Vec<VRow> {
    let w = width.max(8);
    let mut rows = Vec::new();
    for (li, line) in lines.iter().enumerate() {
        if line.is_empty() {
            rows.push(VRow {
                logical: li,
                start: 0,
                end: 0,
                text: String::new(),
            });
            continue;
        }
        let chars: Vec<char> = line.chars().collect();
        let mut start = 0;
        while start < chars.len() {
            let remaining = chars.len() - start;
            let take = remaining.min(w);
            // try not to break mid-word when possible
            let mut end = start + take;
            if end < chars.len() && !chars[end].is_whitespace() {
                if let Some(rel) = chars[start..end]
                    .iter()
                    .rposition(|c| c.is_whitespace())
                {
                    if rel > 0 {
                        end = start + rel + 1;
                    }
                }
            }
            let text: String = chars[start..end].iter().collect();
            rows.push(VRow {
                logical: li,
                start,
                end,
                text,
            });
            start = end;
            // skip leading spaces on continuation
            while start < chars.len() && chars[start].is_whitespace() {
                start += 1;
            }
        }
    }
    if rows.is_empty() {
        rows.push(VRow {
            logical: 0,
            start: 0,
            end: 0,
            text: String::new(),
        });
    }
    rows
}

/// Map logical (line, col) → visual row index + column within that row.
pub fn visual_pos(rows: &[VRow], logical_line: usize, col: usize) -> (usize, usize) {
    for (vi, r) in rows.iter().enumerate() {
        if r.logical == logical_line && col >= r.start && col <= r.end {
            return (vi, col - r.start);
        }
    }
    // past end of last row of that line
    if let Some((vi, r)) = rows
        .iter()
        .enumerate()
        .rev()
        .find(|(_, r)| r.logical == logical_line)
    {
        return (vi, r.end.saturating_sub(r.start));
    }
    (0, 0)
}

/// Map visual (row, col) → logical (line, col).
pub fn logical_pos(rows: &[VRow], visual_row: usize, col: usize) -> (usize, usize) {
    let r = match rows.get(visual_row) {
        Some(r) => r,
        None => return (0, 0),
    };
    let c = r.start + col.min(r.end.saturating_sub(r.start));
    (r.logical, c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_long_line() {
        let lines = vec!["abcdefghijklmnopqrstuvwxyz".into()];
        let rows = wrap_doc(&lines, 10);
        assert!(rows.len() >= 2);
        assert_eq!(rows[0].logical, 0);
    }

    #[test]
    fn up_down_is_one_visual_row() {
        let lines = vec!["one two three four five six seven eight nine ten".into()];
        let rows = wrap_doc(&lines, 12);
        assert!(rows.len() >= 3);
        let (v, c) = visual_pos(&rows, 0, 20);
        let (l, col) = logical_pos(&rows, v.saturating_sub(1), c);
        assert_eq!(l, 0);
        let (v2, _) = visual_pos(&rows, l, col);
        assert_eq!(v2, v.saturating_sub(1));
    }
}
