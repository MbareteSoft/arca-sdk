use arca::auth::{build_login_ticket_request, sign_cms_base64};
use std::fs;

fn main() {
    let cert = fs::read("../certificado/cert.pem").unwrap();
    let key = fs::read("../certificado/key.pem").unwrap();
    
    let ltr = build_login_ticket_request("wsfe");
    println!("=== LoginTicketRequest ===");
    println!("{}", ltr);
    println!();
    
    let cms = sign_cms_base64(&ltr, &cert, &key).unwrap();
    println!("=== CMS Base64 (primeros 100 chars) ===");
    println!("{}...", &cms[..100.min(cms.len())]);
    println!("Total: {} chars", cms.len());
}
