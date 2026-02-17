use crate::constants::namespaces::SOAP_ENVELOPE_NS;
use crate::error::{ArcaError, Result};
use crate::transport::http::HttpClient;

/// Create a SOAP envelope with the specified namespace, prefix, method and body content
pub fn create_soap_envelope(
    namespace: &str,
    prefix: &str,
    method: &str,
    body_content: &str,
) -> String {
    format!(
        r#"<soapenv:Envelope xmlns:soapenv="{}" xmlns:{}="{}">
<soapenv:Header/>
<soapenv:Body>
<{}:{}>
{}
</{}:{}>
</soapenv:Body>
</soapenv:Envelope>"#,
        SOAP_ENVELOPE_NS, prefix, namespace, prefix, method, body_content, prefix, method
    )
}

/// Build a SOAP action URL from namespace and method name
pub fn build_soap_action(namespace: &str, method: &str) -> String {
    format!("{}{}", namespace, method)
}

/// Post a SOAP request using the shared HTTP client
///
/// AFIP services return HTTP 500 for SOAP faults, so we read the body
/// regardless of status code and check for SOAP faults.
pub async fn post_soap(
    client: &HttpClient,
    url: &str,
    soap_xml: &str,
    soap_action: Option<&str>,
) -> Result<String> {
    // SOAPAction header is required even if empty
    let action_header = soap_action.unwrap_or("");

    let req = client
        .client()
        .post(url)
        .header("Content-Type", "text/xml; charset=utf-8")
        .header("SOAPAction", format!("\"{}\"", action_header))
        .body(soap_xml.to_string());

    let response = req.send().await.map_err(ArcaError::Transport)?;

    let status = response.status();
    let body = response.text().await.map_err(ArcaError::Transport)?;

    // AFIP returns HTTP 500 for SOAP faults - check for fault in body
    if !status.is_success() {
        // Check if this is a SOAP fault
        if body.contains("soapenv:Fault") || body.contains("soap:Fault") {
            // Extract fault message for better error reporting
            if let Some(start) = body.find("<faultstring>") {
                if let Some(end) = body.find("</faultstring>") {
                    let fault_msg = &body[start + 13..end];
                    return Err(ArcaError::Xml(format!("SOAP Fault: {}", fault_msg)));
                }
            }
            return Err(ArcaError::Xml(format!(
                "SOAP Fault (HTTP {}): {}",
                status,
                &body[..body.len().min(500)]
            )));
        }
        // Not a SOAP fault, return HTTP error
        return Err(ArcaError::Xml(format!(
            "HTTP error {}: {}",
            status,
            &body[..body.len().min(200)]
        )));
    }

    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_soap_envelope() {
        let envelope = create_soap_envelope(
            "http://example.com/",
            "ex",
            "TestMethod",
            "<ex:Param>value</ex:Param>",
        );

        assert!(envelope.contains("xmlns:soapenv="));
        assert!(envelope.contains("xmlns:ex=\"http://example.com/\""));
        assert!(envelope.contains("<ex:TestMethod>"));
        assert!(envelope.contains("<ex:Param>value</ex:Param>"));
        assert!(envelope.contains("</ex:TestMethod>"));
    }

    #[test]
    fn test_build_soap_action() {
        let action = build_soap_action("http://example.com/", "TestMethod");
        assert_eq!(action, "http://example.com/TestMethod");
    }
}
