use crate::fly::*;
use std::{fs, io, path::Path};
use rcgen::{Certificate, CertificateParams};
use ring::rand::SecureRandom;
use rustls::internal::pemfile;
use std::io::prelude::*;

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt};

pub struct CertPair {
    pub certs: Vec<rustls::Certificate>,
    pub key: rustls::PrivateKey,
}

pub fn get_certs() -> Try<CertPair> {
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
        let gen_cert = Certificate::from_params(params);
        fs::File::create(&cert_path)?
            .write_all(gen_cert.serialize_pem().as_bytes())?;
        write_secret(
            &key_path, gen_cert.serialize_private_key_pem().as_bytes(),
        )?;
    }
    Ok(CertPair{
        certs: load_certs(cert_path)?, key: load_private_key(key_path)?,
    })
}

// Load public certificate from file.
fn load_certs<P: AsRef<Path>>(filename: P) -> Try<Vec<rustls::Certificate>>
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
fn load_private_key<P: AsRef<Path>>(filename: P) -> Try<rustls::PrivateKey>
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

fn write_secret<P: AsRef<Path>>(path: P, buf: &[u8]) -> Try {
    let mut options = fs::OpenOptions::new();
    options.create_new(true);
    #[cfg(unix)] options.mode(0o600);
    options.write(true);
    let mut stream = options.open(path)?;
    stream.write_all(buf)?;
    Ok(())
}
