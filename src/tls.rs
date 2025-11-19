use std::{
    io::{
        self, Cursor,
        ErrorKind::{InvalidData, InvalidInput},
    },
    sync::Arc,
};

use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::{ServerConfig, version::TLS13};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};

pub fn tls_config(cert_path: &str, key_path: &str) -> io::Result<Arc<ServerConfig>> {
    let certs = load_certs(cert_path)?;
    let key = load_private_key(key_path)?;

    let mut config = ServerConfig::builder_with_protocol_versions(&[&TLS13])
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| io::Error::new(InvalidInput, e.to_string()))?;

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Ok(Arc::new(config))
}

fn load_certs(path: &str) -> io::Result<Vec<CertificateDer<'static>>> {
    let input = std::fs::read(path)?;
    let mut cursor = Cursor::new(input);

    let mut out = Vec::new();
    for cert in certs(&mut cursor) {
        out.push(CertificateDer::from(cert?));
    }

    Ok(out)
}

fn load_private_key(path: &str) -> io::Result<PrivateKeyDer<'static>> {
    let input = std::fs::read(path)?;

    let mut cursor = Cursor::new(&input);
    for key in pkcs8_private_keys(&mut cursor) {
        let pk = key?;
        return Ok(PrivateKeyDer::from(pk));
    }

    let mut cursor = Cursor::new(&input);
    for key in rsa_private_keys(&mut cursor) {
        let rk = key?;
        return Ok(PrivateKeyDer::from(rk));
    }

    Err(io::Error::new(InvalidData, "no valid private key found"))
}
