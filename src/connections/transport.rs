
#[derive(Debug, Clone)]
pub enum TlsConfig {
    Simple {
        ca: Vec<u8>,
        alpn: Option<Vec<Vec<u8>>>,
        client_auth: Option<(Vec<u8>, PrivateKey)>,
    },
    // Rustls(Arc<ClientConfig>),
    SimpleNative {
        ca: Vec<u8>,
        der: Vec<u8>,
        password: String,
    },
}

#[derive(Debug, Clone)]
pub enum PrivateKey {
    RSA(Vec<u8>),
    ECC(Vec<u8>),
}