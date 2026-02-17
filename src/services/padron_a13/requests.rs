// SOAP request builders for ws_sr_padron_a13

const NS: &str = "http://a13.soap.ws.server.puc.sr/";

/// Build dummy request
pub fn build_dummy_req() -> String {
    format!(
        r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:a13="{NS}">
<soapenv:Header/>
<soapenv:Body>
<a13:dummy/>
</soapenv:Body>
</soapenv:Envelope>"#
    )
}

/// Build getPersona request
pub fn build_get_persona_req(token: &str, sign: &str, cuit_representada: u64, id_persona: u64) -> String {
    format!(
        r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:a13="{NS}">
<soapenv:Header/>
<soapenv:Body>
<a13:getPersona>
<token>{token}</token>
<sign>{sign}</sign>
<cuitRepresentada>{cuit_representada}</cuitRepresentada>
<idPersona>{id_persona}</idPersona>
</a13:getPersona>
</soapenv:Body>
</soapenv:Envelope>"#
    )
}

/// Build getIdPersonaListByDocumento request
pub fn build_get_id_persona_list_req(
    token: &str,
    sign: &str,
    cuit_representada: u64,
    tipo_documento: &str,
    numero_documento: &str,
) -> String {
    format!(
        r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:a13="{NS}">
<soapenv:Header/>
<soapenv:Body>
<a13:getIdPersonaListByDocumento>
<token>{token}</token>
<sign>{sign}</sign>
<cuitRepresentada>{cuit_representada}</cuitRepresentada>
<documento>
<tipo>{tipo_documento}</tipo>
<numero>{numero_documento}</numero>
</documento>
</a13:getIdPersonaListByDocumento>
</soapenv:Body>
</soapenv:Envelope>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_dummy_req() {
        let req = build_dummy_req();
        assert!(req.contains("<a13:dummy/>"));
    }

    #[test]
    fn test_build_get_persona_req() {
        let req = build_get_persona_req("TOKEN", "SIGN", 20123456789, 20987654321);
        assert!(req.contains("<token>TOKEN</token>"));
        assert!(req.contains("<sign>SIGN</sign>"));
        assert!(req.contains("<cuitRepresentada>20123456789</cuitRepresentada>"));
        assert!(req.contains("<idPersona>20987654321</idPersona>"));
    }
}
