pub fn sanitize_terminal_text(input: &str) -> String {
    input
        .chars()
        .filter(|character| !character.is_control() || matches!(*character, '\n' | '\t'))
        .filter(|character| {
            !matches!(
                *character,
                '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
            )
        })
        .collect()
}

pub fn truncate_chars(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    if max_chars == 0 {
        return String::new();
    }

    let mut truncated: String = input.chars().take(max_chars - 1).collect();
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_terminal_controls_but_keeps_layout_characters() {
        let value = "hello\u{1b}[31m\nworld\t\u{202e}";
        assert_eq!(sanitize_terminal_text(value), "hello[31m\nworld\t");
    }

    #[test]
    fn truncates_by_character() {
        assert_eq!(truncate_chars("你好世界", 3), "你好…");
        assert_eq!(truncate_chars("short", 10), "short");
    }
}
