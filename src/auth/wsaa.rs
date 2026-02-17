use crate::auth::ticket::TicketAcceso;
use crate::constants::defaults::{LOGIN_TICKET_EXP_OFFSET_MINUTES, LOGIN_TICKET_GEN_OFFSET_MINUTES};
use crate::constants::namespaces::{wsaa, SOAP_ENVELOPE_NS};
use crate::error::{ArcaError, Result};
use crate::transport::{post_soap, HttpClient};
use crate::xml::extract_tag_text;
use chrono::{Duration, Utc};

/// Build the LoginTicketRequest XML for WSAA authentication
pub fn build_login_ticket_request(service: &str) -> String {
    let now = Utc::now();
    let uid = now.timestamp();
    let gen_time = now - Duration::minutes(LOGIN_TICKET_GEN_OFFSET_MINUTES);
    let exp = now + Duration::minutes(LOGIN_TICKET_EXP_OFFSET_MINUTES);

    // AFIP expects ISO 8601 format with timezone: YYYY-MM-DDTHH:MM:SS.sss-00:00
    let format_time = |t: chrono::DateTime<Utc>| -> String {
        t.format("%Y-%m-%dT%H:%M:%S.000-00:00").to_string()
    };

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<loginTicketRequest version="1.0">
<header>
<uniqueId>{}</uniqueId>
<generationTime>{}</generationTime>
<expirationTime>{}</expirationTime>
</header>
<service>{}</service>
</loginTicketRequest>"#,
        uid,
        format_time(gen_time),
        format_time(exp),
        service
    )
}

/// Login to WSAA using a CMS-signed ticket request
pub async fn wsaa_login(
    http_client: &HttpClient,
    wsaa_url: &str,
    cms_b64: &str,
) -> Result<TicketAcceso> {
    // Use the exact same format that works with curl
    let soap = format!(
        r#"<soapenv:Envelope xmlns:soapenv="{}" xmlns:wsaa="{}">
<soapenv:Header/>
<soapenv:Body>
<wsaa:loginCms>
<wsaa:in0>{}</wsaa:in0>
</wsaa:loginCms>
</soapenv:Body>
</soapenv:Envelope>"#,
        SOAP_ENVELOPE_NS,
        wsaa::NAMESPACE,
        cms_b64,
    );

    let resp = post_soap(http_client, wsaa_url, &soap, None).await?;
    let ta_xml = extract_tag_text(&resp, "loginCmsReturn")
        .map_err(|e| ArcaError::Xml(format!("Failed to find loginCmsReturn in SOAP response: {}", e)))?;

    Ok(TicketAcceso {
        token: extract_tag_text(&ta_xml, "token")
            .map_err(|e| ArcaError::Xml(format!("Tag 'token' not found in loginCmsReturn: {}", e)))?,
        sign: extract_tag_text(&ta_xml, "sign")
            .map_err(|e| ArcaError::Xml(format!("Tag 'sign' not found in loginCmsReturn: {}", e)))?,
        expiration_time: extract_tag_text(&ta_xml, "expirationTime")
            .map_err(|e| ArcaError::Xml(format!("Tag 'expirationTime' not found in loginCmsReturn: {}", e)))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_login_ticket_request() {
        let ltr = build_login_ticket_request("wsfe");
        assert!(ltr.contains("<service>wsfe</service>"));
        assert!(ltr.contains("<uniqueId>"));
        assert!(ltr.contains("<generationTime>"));
        assert!(ltr.contains("<expirationTime>"));
        assert!(ltr.contains("loginTicketRequest"));
    }

    #[test]
    fn test_build_login_ticket_request_different_service() {
        let ltr = build_login_ticket_request("ws_sr_padron_a13");
        assert!(ltr.contains("<service>ws_sr_padron_a13</service>"));
    }
}
