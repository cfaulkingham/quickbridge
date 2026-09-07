use std::path::Path;

/// Take a client-supplied filename and turn it into a single path segment
/// that cannot escape the destination directory or trip a UI/exec sink.
pub fn sanitize_filename(raw: &str) -> String {
    let name = raw
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(raw)
        .trim()
        .trim_end_matches(['.', ' ']);

    let mut out = String::new();
    for ch in name.chars() {
        if is_forbidden_char(ch) {
            continue;
        }
        if (ch == '.' || ch == '-') && out.is_empty() {
            continue;
        }
        out.push(ch);
        if out.len() >= 180 {
            break;
        }
    }

    let out = out.trim().trim_end_matches(['.', ' ']).to_string();
    if out.is_empty() || out == "." || out == ".." {
        return "upload.bin".to_string();
    }
    neutralize_extension(&out)
}

fn is_forbidden_char(ch: char) -> bool {
    if ch.is_control() {
        return true;
    }
    if ch.is_whitespace() && ch != ' ' {
        return true;
    }
    if "/\\<>:\"|?*&".contains(ch) {
        return true;
    }
    matches!(
        ch,
        '\u{200E}'
            | '\u{200F}'
            | '\u{202A}'
            | '\u{202B}'
            | '\u{202C}'
            | '\u{202D}'
            | '\u{202E}'
            | '\u{2066}'
            | '\u{2067}'
            | '\u{2068}'
            | '\u{2069}'
            | '\u{FEFF}'
    )
}

fn neutralize_extension(name: &str) -> String {
    let ext = Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    const BAD: &[&str] = &[
        "desktop",
        "lnk",
        "url",
        "executable",
        "appimage",
        "com",
        "pif",
        "scf",
        "search-ms",
    ];
    if !BAD.contains(&ext.as_str()) {
        return name.to_string();
    }
    let stem = Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("upload");
    format!("{stem}.bin")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_paths_and_controls() {
        assert_eq!(sanitize_filename("../../etc/passwd"), "passwd");
        assert_eq!(sanitize_filename("C:\\\\Windows\\\\photo.jpg"), "photo.jpg");
        assert_eq!(sanitize_filename(".."), "upload.bin");
        assert_eq!(sanitize_filename(""), "upload.bin");
        assert_eq!(sanitize_filename(".hidden"), "hidden");
        assert_eq!(sanitize_filename("notes\n.pdf"), "notes.pdf");
        assert_eq!(sanitize_filename("ok document.pdf"), "ok document.pdf");
        assert_eq!(sanitize_filename("-evil.pdf"), "evil.pdf");
        assert_eq!(sanitize_filename("a&b<img>.txt"), "abimg.txt");
        assert_eq!(sanitize_filename("notes.desktop"), "notes.bin");
        assert_eq!(sanitize_filename("x\u{202E}gpj.exe"), "xgpj.exe");
    }
}
