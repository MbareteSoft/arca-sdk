//! Utilidades de parseo XML para respuestas AFIP
//!
//! Este módulo provee funciones para extraer datos de respuestas XML SOAP.

use anyhow::{anyhow, Result};
use quick_xml::{escape::resolve_predefined_entity, events::Event, Reader};
use std::str::FromStr;

/// Extract text content from a tag (returns error if not found).
///
/// Handles `Event::GeneralRef` (quick-xml 0.39+) to properly resolve
/// entity references like `&lt;`, `&gt;`, `&amp;` in text content.
pub fn extract_tag_text(xml: &str, tag_name: &str) -> Result<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_tag = false;
    let mut collected = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref().ends_with(tag_name.as_bytes()) => {
                in_tag = true;
                collected.clear();
            }
            Ok(Event::Text(e)) => {
                if in_tag {
                    collected.push_str(&String::from_utf8_lossy(e.as_ref()));
                }
            }
            Ok(Event::GeneralRef(e)) => {
                if in_tag {
                    let ref_name = String::from_utf8_lossy(e.as_ref());
                    if let Some(resolved) = resolve_predefined_entity(&ref_name) {
                        collected.push_str(resolved);
                    } else if let Ok(Some(ch)) = e.resolve_char_ref() {
                        collected.push(ch);
                    }
                }
            }
            Ok(Event::End(e)) if e.name().as_ref().ends_with(tag_name.as_bytes()) => {
                if in_tag {
                    return Ok(collected);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!(e)),
            _ => {}
        }
        buf.clear();
    }
    Err(anyhow!("Tag {} no encontrado", tag_name))
}

/// Extract all XML blocks matching a container tag name
/// Returns a vector of XML strings for each container element
pub fn extract_all_blocks(xml: &str, container_tag: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut depth = 0;
    let mut current_block = String::new();
    let mut in_container = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let qname = e.name();
                let name_bytes = qname.as_ref();
                if name_bytes.ends_with(container_tag.as_bytes()) {
                    in_container = true;
                    depth = 1;
                    current_block.clear();
                } else if in_container {
                    depth += 1;
                    // Reconstruct the tag
                    let name = String::from_utf8_lossy(name_bytes);
                    current_block.push('<');
                    current_block.push_str(&name);
                    current_block.push('>');
                }
            }
            Ok(Event::Text(e)) if in_container => {
                current_block.push_str(&String::from_utf8_lossy(e.as_ref()));
            }
            Ok(Event::GeneralRef(e)) if in_container => {
                let ref_name = String::from_utf8_lossy(e.as_ref());
                if let Some(resolved) = resolve_predefined_entity(&ref_name) {
                    current_block.push_str(resolved);
                } else if let Ok(Some(ch)) = e.resolve_char_ref() {
                    current_block.push(ch);
                }
            }
            Ok(Event::End(e)) => {
                if in_container {
                    let qname = e.name();
                    let name_bytes = qname.as_ref();
                    if name_bytes.ends_with(container_tag.as_bytes()) && depth == 1 {
                        results.push(current_block.clone());
                        in_container = false;
                    } else {
                        depth -= 1;
                        let name = String::from_utf8_lossy(name_bytes);
                        current_block.push_str("</");
                        current_block.push_str(&name);
                        current_block.push('>');
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    results
}

/// Extract text from a simple inline XML block (reconstructed from extract_all_blocks)
pub fn extract_inline_text(block: &str, tag_name: &str) -> Option<String> {
    // Simple regex-like extraction for inline blocks
    let start_pattern = format!("<{}>", tag_name);
    let end_pattern = format!("</{}>", tag_name);

    if let Some(start_idx) = block.find(&start_pattern) {
        let content_start = start_idx + start_pattern.len();
        if let Some(end_idx) = block[content_start..].find(&end_pattern) {
            return Some(block[content_start..content_start + end_idx].to_string());
        }
    }
    None
}

// ============================================================================
// Generic parsing helpers - reduce boilerplate in service implementations
// ============================================================================

/// Extract and parse a tag value to any type that implements FromStr.
/// Returns error if tag not found or parsing fails.
///
/// # Example
/// ```ignore
/// let cbte_nro: u64 = parse_tag(&xml, "CbteNro")?;
/// let imp_total: f64 = parse_tag(&xml, "ImpTotal")?;
/// ```
pub fn parse_tag<T: FromStr>(xml: &str, tag_name: &str) -> Result<T>
where
    T::Err: std::fmt::Display,
{
    let text = extract_tag_text(xml, tag_name)?;
    text.parse::<T>()
        .map_err(|e| anyhow!("Failed to parse '{}' from tag {}: {}", text, tag_name, e))
}

/// Extract and parse a tag value, returning None if tag not found.
/// Returns error only if tag exists but parsing fails.
pub fn parse_tag_optional<T: FromStr>(xml: &str, tag_name: &str) -> Result<Option<T>>
where
    T::Err: std::fmt::Display,
{
    match extract_tag_text(xml, tag_name) {
        Ok(text) => {
            let value = text.parse::<T>()
                .map_err(|e| anyhow!("Failed to parse '{}' from tag {}: {}", text, tag_name, e))?;
            Ok(Some(value))
        }
        Err(_) => Ok(None),
    }
}

/// Extract tag text, returning None if not found (no error)
pub fn extract_tag_text_optional(xml: &str, tag_name: &str) -> Option<String> {
    extract_tag_text(xml, tag_name).ok()
}

/// Extract and parse a tag from a block (used with extract_all_blocks)
pub fn parse_inline<T: FromStr>(block: &str, tag_name: &str) -> Option<T> {
    extract_inline_text(block, tag_name)?
        .parse::<T>()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag_i32() {
        let xml = r#"<Response><CbteNro>123</CbteNro></Response>"#;
        let result: i32 = parse_tag(xml, "CbteNro").unwrap();
        assert_eq!(result, 123);
    }

    #[test]
    fn test_parse_tag_f64() {
        let xml = r#"<Response><ImpTotal>1234.56</ImpTotal></Response>"#;
        let result: f64 = parse_tag(xml, "ImpTotal").unwrap();
        assert!((result - 1234.56).abs() < 0.001);
    }

    #[test]
    fn test_parse_tag_optional_missing() {
        let xml = r#"<Response><Other>123</Other></Response>"#;
        let result: Option<i32> = parse_tag_optional(xml, "CbteNro").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_inline() {
        let block = "<Id>5</Id><Desc>Test</Desc>";
        let id: i32 = parse_inline(block, "Id").unwrap();
        assert_eq!(id, 5);
    }

    #[test]
    fn test_extract_tag_text_unescapes_html_entities() {
        // Simulates WSAA response: loginCmsReturn contains HTML-encoded XML
        let xml = r#"<soap:Body><loginCmsReturn>&lt;loginTicketResponse&gt;&lt;credentials&gt;&lt;token&gt;ABC123&lt;/token&gt;&lt;sign&gt;XYZ789&lt;/sign&gt;&lt;/credentials&gt;&lt;header&gt;&lt;expirationTime&gt;2026-02-17T22:00:00.000-00:00&lt;/expirationTime&gt;&lt;/header&gt;&lt;/loginTicketResponse&gt;</loginCmsReturn></soap:Body>"#;

        // First extraction should decode HTML entities
        let inner = extract_tag_text(xml, "loginCmsReturn").unwrap();
        assert!(inner.contains("<token>ABC123</token>"), "inner XML should be decoded: {}", inner);

        // Second extraction from decoded XML should find tags
        let token = extract_tag_text(&inner, "token").unwrap();
        assert_eq!(token, "ABC123");

        let sign = extract_tag_text(&inner, "sign").unwrap();
        assert_eq!(sign, "XYZ789");

        let exp = extract_tag_text(&inner, "expirationTime").unwrap();
        assert_eq!(exp, "2026-02-17T22:00:00.000-00:00");
    }
}
