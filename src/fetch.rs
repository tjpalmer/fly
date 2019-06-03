use crate::fly::*;
use futures::{Future, Stream};
use hyper::{Body, Uri, client::Client};
use rustls::{
    Certificate, ClientConfig, RootCertStore, ServerCertVerifier,
    ServerCertVerified, TLSError,
};
use std::str::FromStr;
use std::sync::Arc;

struct InsecureVerifier {}

impl ServerCertVerifier for InsecureVerifier {
    fn verify_server_cert(
        &self,
        _roots: &RootCertStore,
        _presented_certs: &[Certificate],
        _dns_name: webpki::DNSNameRef,
        _ocsp_response: &[u8],
    ) -> Result<ServerCertVerified, TLSError> {
        Ok(ServerCertVerified::assertion())
    }
}

pub fn fetch(uri: &str) -> Try {
    let mut http = hyper::client::HttpConnector::new(1);
    http.enforce_http(false);
    let mut tls = ClientConfig::new();
    tls.dangerous().set_certificate_verifier(Arc::new(InsecureVerifier {}));
    let https = hyper_rustls::HttpsConnector::from((http, tls));
    let client: Client<_, Body> = Client::builder().build(https);
    let uri_obj = Uri::from_str(uri)?;
    let future = futures::future::ok(client)
        .and_then(|client| client.get(uri_obj))
        .inspect(|response| {
            println!("Status: {}", response.status());
            println!("Headers: {:#?}", response.headers());
        })
        .and_then(|response| response.into_body().concat2())
        .inspect(|body| {
            println!("Body:\n{}", String::from_utf8_lossy(&body));
        });
    // TODO Some central runtime for all to share?
    let rt = tokio::runtime::Runtime::new()?;
    // TODO No such synchronizing calls around here?
    rt.block_on_all(future)?;
    Ok(())
}
