/// Strip control characters and ANSI escape sequences from git-derived text before display.
pub fn display_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            skip_ansi_sequence(&mut chars);
            continue;
        }
        if ch.is_control() && ch != '\t' && ch != '\n' {
            continue;
        }
        out.push(ch);
    }
    out
}

fn skip_ansi_sequence<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) {
    for ch in chars.by_ref() {
        if ch.is_ascii_alphabetic() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ansi_and_control_chars() {
        let raw = "Alice\x1b[31mred\x1b[0m\x07";
        assert_eq!(display_text(raw), "Alicered");
    }

    #[test]
    fn keeps_tabs_and_newlines() {
        assert_eq!(display_text("a\tb\nc"), "a\tb\nc");
    }
}
