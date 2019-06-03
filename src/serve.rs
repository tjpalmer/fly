use crate::fly::*;
use futures::{Stream, future};
use hyper::{
    Body, Method, Request, Response, Server, StatusCode,
    rt::Future, service::service_fn,
};
use std::{io, sync};
use tokio_rustls::TlsAcceptor;

pub fn run_server(cert_pair: &CertPair) -> Try {
    let port = "1337";
    let addr = format!("0.0.0.0:{}", port)
        .parse()
        .map_err(|e| error(format!("{}", e)))?;

    // Build TLS configuration.
    // TODO How to make certs?
    let tls_cfg = {
        // Do not use client certificate authentication.
        let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
        // Select a certificate to use.
        cfg.set_single_cert(cert_pair.certs.clone(), cert_pair.key.clone())
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
    let rt = tokio::runtime::Runtime::new()?;
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
