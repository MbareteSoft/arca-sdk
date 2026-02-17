use std::fs;

fn main() {
    let cert = fs::read("../certificado/cert.pem").unwrap();
    let key = fs::read("../certificado/key.pem").unwrap();
    
    let ltr = r#"<?xml version="1.0" encoding="UTF-8"?>
<loginTicketRequest version="1.0">
<header>
<uniqueId>999999999</uniqueId>
<generationTime>2025-12-26T14:10:15.000-00:00</generationTime>
<expirationTime>2025-12-26T14:30:15.000-00:00</expirationTime>
</header>
<service>wsfe</service>
</loginTicketRequest>"#;
    
    use openssl::cms::{CMSOptions, CmsContentInfo};
    use openssl::pkey::PKey;
    use openssl::x509::X509;
    
    let cert = X509::from_pem(&cert).unwrap();
    let key = PKey::private_key_from_pem(&key).unwrap();
    
    let cms = CmsContentInfo::sign(
        Some(&cert),
        Some(&key),
        None,
        Some(ltr.as_bytes()),
        CMSOptions::empty(),
    ).unwrap();
    
    let der = cms.to_der().unwrap();
    std::fs::write("/tmp/rust_cms.der", &der).unwrap();
    println!("CMS Rust: {} bytes", der.len());
}
