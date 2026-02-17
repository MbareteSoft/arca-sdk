//! Debug CAE request to see ARCA response
//!
//! cargo run --example debug_cae

use std::fs;

const CUIT: u64 = 20123456781;
const PTO_VTA: i32 = 1;
const CBTE_TIPO: i32 = 6;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Debug CAE Request ===\n");

    // Load certs
    let cert = fs::read("../certificado/cert.pem")?;
    let key = fs::read("../certificado/key.pem")?;

    // Login first
    println!("1. Login WSAA...");
    let ltr = arca::auth::build_login_ticket_request("wsfe");
    let cms = arca::auth::sign_cms_base64(&ltr, &cert, &key)?;

    let http_client = arca::transport::HttpClient::new()?;
    let wsaa_url = "https://wsaahomo.afip.gov.ar/ws/services/LoginCms";

    let ta = arca::auth::wsaa_login(&http_client, wsaa_url, &cms).await?;
    println!("   Token: {}...", &ta.token[..50.min(ta.token.len())]);
    println!("   Sign: {}...", &ta.sign[..50.min(ta.sign.len())]);

    // Get last voucher
    println!("\n2. Getting last voucher...");
    let ultimo_soap = format!(
        r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:ar="http://ar.gov.afip.dif.FEV1/">
<soapenv:Header/>
<soapenv:Body>
<ar:FECompUltimoAutorizado>
<ar:Auth>
<ar:Token>{}</ar:Token>
<ar:Sign>{}</ar:Sign>
<ar:Cuit>{}</ar:Cuit>
</ar:Auth>
<ar:PtoVta>{}</ar:PtoVta>
<ar:CbteTipo>{}</ar:CbteTipo>
</ar:FECompUltimoAutorizado>
</soapenv:Body>
</soapenv:Envelope>"#,
        ta.token, ta.sign, CUIT, PTO_VTA, CBTE_TIPO
    );

    let wsfev1_url = "https://wswhomo.afip.gov.ar/wsfev1/service.asmx";
    let resp = arca::transport::post_soap(&http_client, wsfev1_url, &ultimo_soap, Some("http://ar.gov.afip.dif.FEV1/FECompUltimoAutorizado")).await?;

    let ultimo: i32 = arca::xml::extract_tag_text(&resp, "CbteNro")?.parse()?;
    println!("   Último: {}", ultimo);

    let nuevo = (ultimo + 1) as u64;
    println!("   Nuevo: {}", nuevo);

    // Request CAE
    println!("\n3. Requesting CAE...");
    let fecha_hoy = chrono::Local::now().format("%Y%m%d").to_string();

    let cae_soap = format!(
        r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:ar="http://ar.gov.afip.dif.FEV1/">
<soapenv:Header/>
<soapenv:Body>
<ar:FECAESolicitar>
<ar:Auth>
<ar:Token>{}</ar:Token>
<ar:Sign>{}</ar:Sign>
<ar:Cuit>{}</ar:Cuit>
</ar:Auth>
<ar:FeCAEReq>
<ar:FeCabReq>
<ar:CantReg>1</ar:CantReg>
<ar:PtoVta>{}</ar:PtoVta>
<ar:CbteTipo>{}</ar:CbteTipo>
</ar:FeCabReq>
<ar:FeDetReq>
<ar:FECAEDetRequest>
<ar:Concepto>1</ar:Concepto>
<ar:DocTipo>99</ar:DocTipo>
<ar:DocNro>0</ar:DocNro>
<ar:CbteDesde>{}</ar:CbteDesde>
<ar:CbteHasta>{}</ar:CbteHasta>
<ar:CbteFch>{}</ar:CbteFch>
<ar:ImpTotal>121</ar:ImpTotal>
<ar:ImpTotConc>0</ar:ImpTotConc>
<ar:ImpNeto>100</ar:ImpNeto>
<ar:ImpOpEx>0</ar:ImpOpEx>
<ar:ImpTrib>0</ar:ImpTrib>
<ar:ImpIVA>21</ar:ImpIVA>
<ar:MonId>PES</ar:MonId>
<ar:MonCotiz>1</ar:MonCotiz>
<ar:Iva>
<ar:AlicIva>
<ar:Id>5</ar:Id>
<ar:BaseImp>100</ar:BaseImp>
<ar:Importe>21</ar:Importe>
</ar:AlicIva>
</ar:Iva>
</ar:FECAEDetRequest>
</ar:FeDetReq>
</ar:FeCAEReq>
</ar:FECAESolicitar>
</soapenv:Body>
</soapenv:Envelope>"#,
        ta.token, ta.sign, CUIT, PTO_VTA, CBTE_TIPO, nuevo, nuevo, fecha_hoy
    );

    println!("\n   SOAP Request:\n{}\n", &cae_soap[..500]);

    let resp = arca::transport::post_soap(&http_client, wsfev1_url, &cae_soap, Some("http://ar.gov.afip.dif.FEV1/FECAESolicitar")).await?;

    println!("\n   SOAP Response:\n{}\n", resp);

    // Try to extract CAE
    match arca::xml::extract_tag_text(&resp, "CAE") {
        Ok(cae) => println!("   CAE: {}", cae),
        Err(e) => println!("   No CAE found: {}", e),
    }

    // Try to extract errors
    match arca::xml::extract_tag_text(&resp, "Err") {
        Ok(err) => println!("   Error block: {}", err),
        Err(_) => {}
    }

    match arca::xml::extract_tag_text(&resp, "Msg") {
        Ok(msg) => println!("   Error Msg: {}", msg),
        Err(_) => {}
    }

    Ok(())
}
