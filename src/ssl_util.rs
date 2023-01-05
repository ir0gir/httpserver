/// Returns generated certificate, private key
pub fn create_cert_pair() -> (Vec<u8>, Vec<u8>) {
    use rcgen::generate_simple_self_signed;
    let subject_alt_names = vec!["localhost".to_string()];
    let cert = generate_simple_self_signed(subject_alt_names).unwrap();

    (cert.serialize_pem().unwrap().into_bytes(), cert.serialize_private_key_pem().into_bytes())
}