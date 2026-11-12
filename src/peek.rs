//! The last lines of a tab's screen for the peek card. Free of host calls.

/// Screen text is untrusted: escape sequences and other control or format characters are dropped,
/// tabs become spaces. Returns at most `rows` lines of at most `cols` characters, the last ones with
/// text, trailing blank lines removed.
pub fn tail(screen: &str, rows: usize, cols: usize) -> Vec<String> {
    let mut lines: Vec<String> = strip(screen).lines().map(|l| l.trim_end().chars().take(cols).collect()).collect();
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    let start = lines.len().saturating_sub(rows);
    lines.split_off(start)
}

fn strip(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '\u{1b}' => match it.next() {
                // CSI: parameters and intermediates, then one final byte.
                Some('[') => {
                    while let Some(&n) = it.peek() {
                        it.next();
                        if ('\u{40}'..='\u{7e}').contains(&n) {
                            break;
                        }
                    }
                }
                // OSC, DCS, APC, PM, SOS: up to BEL or ST.
                Some(']' | 'P' | '_' | '^' | 'X') => {
                    while let Some(n) = it.next() {
                        if n == '\u{7}' || (n == '\u{1b}' && it.peek() == Some(&'\\')) {
                            if n == '\u{1b}' {
                                it.next();
                            }
                            break;
                        }
                    }
                }
                _ => {}
            },
            '\n' => out.push('\n'),
            '\t' => out.push_str("    "),
            '\r' => {}
            c if c.is_control() => {}
            '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}' | '\u{e0000}'..='\u{e0fff}' => {}
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_last_lines_with_text() {
        let s = "a\nb\n$ npm test session\n PASS  session.test.ts (14)\n RUNS  refresh.test.ts\n▍\n\n\n";
        assert_eq!(tail(s, 4, 80), ["$ npm test session", " PASS  session.test.ts (14)", " RUNS  refresh.test.ts", "▍"]);
        assert!(tail("", 4, 80).is_empty());
    }

    #[test]
    fn escapes_and_hidden_characters_are_dropped() {
        let s = "\u{1b}[1;32mPASS\u{1b}[0m x\u{1b}]8;;https://evil\u{7}link\u{1b}]8;;\u{1b}\\\r\n\u{202e}ok\tz\u{7}";
        assert_eq!(tail(s, 4, 80), ["PASS xlink", "ok    z"]);
    }

    #[test]
    fn long_lines_are_cut() {
        assert_eq!(tail(&"x".repeat(200), 1, 10), ["x".repeat(10)]);
    }
}
