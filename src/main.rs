//! Simple HTTPS echo service based on hyper-rustls
//!
//! First parameter is the mandatory port to use.
//! Certificate and private key are hardcoded to sample files.
#![deny(warnings)]

extern crate dirs;
extern crate failure;
extern crate futures;
extern crate hyper;
extern crate rcgen;
extern crate ring;
extern crate rustls;
extern crate tokio;
extern crate tokio_rustls;
extern crate tokio_tcp;

use failure::Error;
use futures::future;
use futures::Stream;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use rcgen::{Certificate, CertificateParams};
use ring::rand::SecureRandom;
use rustls::internal::pemfile;
use std::{env, fs, io, sync};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use tokio_rustls::TlsAcceptor;

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt};

struct CertPair {
    cert_path: PathBuf,
    key_path: PathBuf,
}

fn main() -> Result<(), Error> {
    let cert_pair = create_certs()?;
    // Serve an echo service over HTTPS, with proper error handling.
    if let Err(e) = run_server(&cert_pair) {
        eprintln!("FAILED: {}", e);
        std::process::exit(1);
    }
    Ok(())
}

fn write_secret<P: AsRef<Path>>(path: P, buf: &[u8]) -> Result<(), Error> {
    let mut options = fs::OpenOptions::new();
    options.create_new(true);
    #[cfg(unix)] options.mode(0o600);
    options.write(true);
    let mut stream = options.open(path)?;
    stream.write_all(buf)?;
    Ok(())
}

fn create_certs() -> Result<CertPair, Error> {
    let dir = dirs::data_local_dir().unwrap().join("fly");
    let cert_path = dir.join("cert.pem");
    let key_path = dir.join("cert-key.pem");
    if !(cert_path.exists() && key_path.exists()) {
        if !dir.exists() {
            fs::create_dir_all(&dir)?
        }
        println!("{}", cert_path.to_str().unwrap());
        let mut params = CertificateParams::default();
        let mut bytes: [u8; 8] = [0; 8];
        ring::rand::SystemRandom::new().fill(&mut bytes)?;
        params.serial_number = Some(u64::from_ne_bytes(bytes));
        let cert = Certificate::from_params(params);
        fs::File::create(&cert_path)?
            .write_all(cert.serialize_pem().as_bytes())?;
        write_secret(&key_path, cert.serialize_private_key_pem().as_bytes())?;
    }
    Ok(CertPair{cert_path, key_path})
}

fn error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

fn run_server(cert_pair: &CertPair) -> io::Result<()> {
    // First parameter is port number (optional, defaults to 1337)
    let port = match env::args().nth(1) {
        Some(ref p) => p.to_owned(),
        None => "1337".to_owned(),
    };
    let addr = format!("127.0.0.1:{}", port)
        .parse()
        .map_err(|e| error(format!("{}", e)))?;

    // Build TLS configuration.
    // TODO How to make certs?
    let tls_cfg = {
        // Load public certificate.
        let certs = load_certs(&cert_pair.cert_path)?;
        // Load private key.
        let key = load_private_key(&cert_pair.key_path)?;
        // Do not use client certificate authentication.
        let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
        // Select a certificate to use.
        cfg.set_single_cert(certs, key)
            .map_err(|e| error(format!("{}", e)))?;
        sync::Arc::new(cfg)
    };

    // Create a TCP listener via tokio.
    let tcp = tokio_tcp::TcpListener::bind(&addr)?;
    let tls_acceptor = TlsAcceptor::from(tls_cfg);
    // Prepare a long-running future stream to accept and serve cients.
    let tls = tcp.incoming()
        .and_then(move |s| tls_acceptor.accept(s))
        .then(|r| match r {
            Ok(x) => Ok::<_, io::Error>(Some(x)),
            Err(_e) => {
                // println!(concat!(
                //     "[!] Voluntary server halt due to ",
                //     "client-connection error ...",
                // ));
                // Errors could be handled here, instead of server aborting.
                Ok(None)
                // Err(_e)
            }
        })
        .filter_map(|x| x);
    // Build a hyper server, which serves our custom echo service.
    let fut = Server::builder(tls).serve(|| service_fn(echo));

    // Run the future, keep going until an error occurs.
    println!("Starting to serve on https://{}.", addr);
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on_all(fut)
        .map_err(|e| error(format!("{}", e)))?;
    Ok(())
}

// Future result: either a hyper body or an error.
type ResponseFuture =
    Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

// Custom echo service, handling two different routes and a
// catch-all 404 responder.
fn echo(req: Request<Body>) -> ResponseFuture {
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        // Help route.
        (&Method::GET, "/") => {
            *response.body_mut() = Body::from("Try POST /echo - Srsly?\n");
        }
        // Echo service route.
        (&Method::POST, "/echo") => {
            *response.body_mut() = req.into_body();
        }
        // Catch-all 404.
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Box::new(future::ok(response))
}

// Load public certificate from file.
fn load_certs<P: AsRef<Path>>(filename: P)
    -> io::Result<Vec<rustls::Certificate>>
{
    // Open certificate file.
    let certfile = fs::File::open(&filename).map_err(|e| {
        error(format!(
            "failed to open {}: {}", filename.as_ref().to_str().unwrap(), e,
        ))
    })?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    pemfile::certs(&mut reader)
        .map_err(|_| error("failed to load certificate".into()))
}

// Load private key from file.
fn load_private_key<P: AsRef<Path>>(filename: P)
    -> io::Result<rustls::PrivateKey>
{
    // Open keyfile.
    let keyfile = fs::File::open(&filename).map_err(|e| {
        error(format!(
            "failed to open {}: {}", filename.as_ref().to_str().unwrap(), e,
        ))
    })?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = pemfile::pkcs8_private_keys(&mut reader)
        .map_err(|_| error("failed to load private key".into()))?;
    if keys.len() != 1 {
        return Err(error("expected a single private key".into()));
    }
    Ok(keys[0].clone())
}
