//! Generate CMS and save to file for curl testing
//!
//! cargo run --example gen_cms_for_curl

use std::fs;

fn main() -> anyhow::Result<()> {
    let cert = fs::read("../certificado/cert.pem")?;
    let key = fs::read("../certificado/key.pem")?;

    let ltr = arca::auth::build_login_ticket_request("wsfe");
    println!("LTR:\n{}\n", ltr);

    let cms = arca::auth::sign_cms_base64(&ltr, &cert, &key)?;
    println!("CMS length: {} chars", cms.len());

    // Save CMS to file
    fs::write("/tmp/rust_cms_b64.txt", &cms)?;
    println!("CMS saved to /tmp/rust_cms_b64.txt");

    // Create SOAP envelope
    let soap = format!(
        r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:wsaa="http://wsaa.view.sua.dvadac.desein.afip.gov">
<soapenv:Header/>
<soapenv:Body>
<wsaa:loginCms>
<wsaa:in0>{}</wsaa:in0>
</wsaa:loginCms>
</soapenv:Body>
</soapenv:Envelope>"#,
        cms
    );

    fs::write("/tmp/rust_soap.xml", &soap)?;
    println!("SOAP saved to /tmp/rust_soap.xml");

    println!("\nTest with curl:");
    println!(r#"curl -X POST -H "Content-Type: text/xml; charset=utf-8" -H "SOAPAction: \"\"" -d @/tmp/rust_soap.xml https://wsaahomo.afip.gov.ar/ws/services/LoginCms"#);

    Ok(())
}
