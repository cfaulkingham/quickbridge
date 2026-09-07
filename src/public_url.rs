/// Allowlisted public origin for a Cloudflare quick tunnel.
///
/// The API hostname is untrusted input. Only a single-label
/// `https://<sub>.trycloudflare.com` host is accepted before the
/// session token is appended.
pub fn tunnel_origin(raw: &str) -> anyhow::Result<String> {
    let s = raw.trim().trim_end_matches('/');
    let host = s
        .strip_prefix("https://")
        .ok_or_else(|| anyhow::anyhow!("tunnel URL must be https"))?;
    if host.is_empty()
        || host.contains('/')
        || host.contains('\\')
        || host.contains('@')
        || host.contains('?')
        || host.contains('#')
        || host.contains(':')
        || host.contains('[')
        || host.contains(']')
        || host.contains(' ')
        || host.contains('\0')
    {
        anyhow::bail!("tunnel URL is not a bare https host");
    }
    let host = host.to_ascii_lowercase();
    const SUFFIX: &str = ".trycloudflare.com";
    let Some(sub) = host.strip_suffix(SUFFIX) else {
        anyhow::bail!("tunnel host is not trycloudflare.com");
    };
    if sub.is_empty()
        || sub.contains('.')
        || sub.starts_with('-')
        || sub.ends_with('-')
        || !sub
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        || sub.len() > 63
    {
        anyhow::bail!("tunnel host is not a trycloudflare subdomain");
    }
    Ok(format!("https://{host}"))
}

pub fn local_origin(raw: &str, port: u16) -> anyhow::Result<String> {
    let expected = format!("http://127.0.0.1:{port}");
    if raw.trim().trim_end_matches('/') != expected {
        anyhow::bail!("local URL mismatch");
    }
    Ok(expected)
}

pub fn session_url(origin: &str, token: &str) -> anyhow::Result<String> {
    if token.len() != 32 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        anyhow::bail!("session token is malformed");
    }
    Ok(format!("{origin}/s/{token}/"))
}

pub fn proxy_url(origin: &str) -> anyhow::Result<String> {
    if origin.starts_with("https://") {
        let _ = tunnel_origin(origin)?;
    } else if origin.starts_with("http://127.0.0.1:") {
        let rest = origin.trim().trim_end_matches('/');
        if rest.split(':').nth(2).and_then(|p| p.parse::<u16>().ok()).is_none() {
            anyhow::bail!("local URL mismatch");
        }
    } else {
        anyhow::bail!("proxy origin is not allowed");
    }
    Ok(format!("{}/", origin.trim().trim_end_matches('/')))
}

pub fn sanitize_location(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(16)
        .collect();
    if cleaned.is_empty() {
        "edge".to_string()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_trycloudflare() {
        assert_eq!(
            tunnel_origin("https://abc-123.trycloudflare.com/").unwrap(),
            "https://abc-123.trycloudflare.com"
        );
    }

    #[test]
    fn rejects_other_hosts() {
        assert!(tunnel_origin("http://abc.trycloudflare.com").is_err());
        assert!(tunnel_origin("https://evil.com").is_err());
        assert!(tunnel_origin("https://trycloudflare.com").is_err());
        assert!(tunnel_origin("https://a.b.trycloudflare.com").is_err());
        assert!(tunnel_origin("https://user@abc.trycloudflare.com").is_err());
        assert!(tunnel_origin("https://abc.trycloudflare.com/phish").is_err());
    }

    #[test]
    fn session_url_requires_hex_token() {
        let origin = "https://abc.trycloudflare.com";
        let token = "0123456789abcdef0123456789abcdef";
        assert_eq!(
            session_url(origin, token).unwrap(),
            format!("{origin}/s/{token}/")
        );
        assert!(session_url(origin, "nope").is_err());
    }

    #[test]
    fn proxy_url_adds_slash() {
        assert_eq!(
            proxy_url("https://abc-123.trycloudflare.com").unwrap(),
            "https://abc-123.trycloudflare.com/"
        );
        assert_eq!(
            proxy_url("http://127.0.0.1:3456").unwrap(),
            "http://127.0.0.1:3456/"
        );
        assert!(proxy_url("https://evil.com").is_err());
    }
}
