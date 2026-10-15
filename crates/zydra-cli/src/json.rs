//! Keluaran terstruktur untuk `--json`.
//!
//! Ditulis tangan alih-alih memakai crate serialisasi karena bentuknya sederhana dan
//! tetap: satu objek datar per baris, hanya berisi string dan angka. Yang penting adalah
//! pelolosan karakter, dan itu diuji langsung.

/// Meloloskan sebuah string menjadi literal JSON, termasuk tanda kutipnya.
pub fn string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            control if control < '\u{20}' => {
                out.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// Menyusun satu objek JSON datar. Nilai harus sudah diloloskan lewat [`string`] atau
/// berupa angka.
pub fn object(fields: &[(&str, String)]) -> String {
    let body: Vec<String> = fields
        .iter()
        .map(|(name, value)| format!("{}:{}", string(name), value))
        .collect();
    format!("{{{}}}", body.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_string_is_quoted() {
        assert_eq!(string("vault.zip"), "\"vault.zip\"");
    }

    #[test]
    fn quotes_and_backslashes_are_escaped() {
        assert_eq!(string("a\"b"), "\"a\\\"b\"");
        assert_eq!(string("C:\\tmp"), "\"C:\\\\tmp\"");
    }

    #[test]
    fn control_characters_are_escaped() {
        assert_eq!(string("a\nb"), "\"a\\nb\"");
        assert_eq!(string("a\tb"), "\"a\\tb\"");
        assert_eq!(string("\u{1}"), "\"\\u0001\"");
    }

    #[test]
    fn non_ascii_text_is_left_alone() {
        assert_eq!(string("kunci rahasia"), "\"kunci rahasia\"");
    }

    #[test]
    fn an_object_joins_its_fields() {
        let line = object(&[("kind", string("progress")), ("tried", "42".to_string())]);
        assert_eq!(line, "{\"kind\":\"progress\",\"tried\":42}");
    }

    #[test]
    fn an_empty_object_is_still_valid() {
        assert_eq!(object(&[]), "{}");
    }

    #[test]
    fn a_password_with_quotes_stays_valid_json() {
        let line = object(&[("password", string("say \"hi\"\\"))]);
        assert_eq!(line, "{\"password\":\"say \\\"hi\\\"\\\\\"}");
    }
}
