/// Build the standard Auth XML block used by most AFIP services
///
/// # Arguments
/// * `prefix` - The XML namespace prefix (e.g., "ar" for WSFEv1)
/// * `token` - The authentication token from WSAA
/// * `sign` - The signature from WSAA
/// * `cuit` - The CUIT number
pub fn build_auth_block(prefix: &str, token: &str, sign: &str, cuit: u64) -> String {
    format!(
        r#"<{0}:Auth>
<{0}:Token>{1}</{0}:Token>
<{0}:Sign>{2}</{0}:Sign>
<{0}:Cuit>{3}</{0}:Cuit>
</{0}:Auth>"#,
        prefix, token, sign, cuit
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_auth_block() {
        let auth = build_auth_block("ar", "my_token", "my_sign", 20111111112);

        assert!(auth.contains("<ar:Auth>"));
        assert!(auth.contains("<ar:Token>my_token</ar:Token>"));
        assert!(auth.contains("<ar:Sign>my_sign</ar:Sign>"));
        assert!(auth.contains("<ar:Cuit>20111111112</ar:Cuit>"));
        assert!(auth.contains("</ar:Auth>"));
    }

    #[test]
    fn test_build_auth_block_different_prefix() {
        let auth = build_auth_block("ws", "token", "sign", 12345678901);

        assert!(auth.contains("<ws:Auth>"));
        assert!(auth.contains("<ws:Token>"));
        assert!(auth.contains("</ws:Auth>"));
    }
}
