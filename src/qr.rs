use qrcode::{Color, QrCode};

const QUIET_ZONE: usize = 4;
/// Version-40 QR is 177 modules; quiet zone adds 8. Cap the matrix the
/// panel will turn into a Repeater so a hostile helper cannot OOM the shell.
pub const MAX_QR_SIZE: usize = 200;

/// Encode `payload` as a square 0/1 module matrix, including the QR quiet
/// zone so the Omarchy panel can paint only the dark modules on a white
/// rounded canvas without eating into the code.
pub fn matrix_for(payload: &str) -> anyhow::Result<Vec<String>> {
    let code = QrCode::new(payload.as_bytes())?;
    let width = code.width();
    let size = width + QUIET_ZONE * 2;
    if size > MAX_QR_SIZE {
        anyhow::bail!("QR code is too large to render");
    }
    let mut rows = Vec::with_capacity(size);
    for y in 0..size {
        let mut row = String::with_capacity(size);
        for x in 0..size {
            let dark = x >= QUIET_ZONE
                && y >= QUIET_ZONE
                && x < QUIET_ZONE + width
                && y < QUIET_ZONE + width
                && code[(x - QUIET_ZONE, y - QUIET_ZONE)] == Color::Dark;
            row.push(if dark { '1' } else { '0' });
        }
        rows.push(row);
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_is_square_bits() {
        let rows = matrix_for("https://example.trycloudflare.com/s/abc").unwrap();
        assert!(!rows.is_empty());
        let size = rows.len();
        for row in &rows {
            assert_eq!(row.len(), size);
            assert!(row.chars().all(|c| c == '0' || c == '1'));
        }
        assert!(rows.iter().any(|row| row.contains('1')));
        assert!(rows[0].chars().all(|c| c == '0'));
        assert!(rows[size - 1].chars().all(|c| c == '0'));
    }
}
