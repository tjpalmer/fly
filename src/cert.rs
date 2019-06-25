use crate::fly::*;
use std::{fs, io, path::Path};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DistinguishedName, DnType,
    IsCa,
};
use ring::rand::SecureRandom;
use rustls::internal::pemfile;
use std::convert::TryInto;
use std::io::prelude::*;
use untrusted::Input;
use webpki::{
    TLSServerTrustAnchors, trust_anchor_util::cert_der_as_trust_anchor,
};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt};

pub struct CertPair {
    pub certs: Vec<rustls::Certificate>,
    pub key: rustls::PrivateKey,
}

struct CertPathPair<P: AsRef<Path>> {
    cert_path: P,
    key_path: P,
}

pub fn get_certs(name: &str, signer: Option<&CertPair>) -> Try<CertPair> {
    // TODO Change to config dir?
    let dir = dirs::data_local_dir().unwrap().join("fly");
    let cert_path = dir.join(format!("{}-cert.pem", name));
    let key_path = dir.join(format!("{}-key.pem", name));
    if !(cert_path.exists() && key_path.exists()) {
        if !dir.exists() {
            fs::create_dir_all(&dir)?
        }
        println!("{}", cert_path.to_str().unwrap());
        match signer {
            Some(parent) => make_node_cert(
                &parent,
                &CertPathPair {
                    cert_path: &cert_path,
                    key_path: &key_path,
                }
            )?,
            None => make_ca_cert(&CertPathPair {
                cert_path: &cert_path,
                key_path: &key_path,
            })?,
        }
    }
    Ok(CertPair{
        certs: load_certs(cert_path)?, key: load_private_key(key_path)?,
    })
}

// Load public certificate from file.
fn load_certs<P: AsRef<Path>>(filename: P) -> Try<Vec<rustls::Certificate>> {
    // Open certificate file.
    let certfile = fs::File::open(&filename)?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    pemfile::certs(&mut reader)
        .map_err(|_| error("failed to load certificate"))
}

// Load private key from file.
fn load_private_key<P: AsRef<Path>>(filename: P) -> Try<rustls::PrivateKey> {
    // Open keyfile.
    let keyfile = fs::File::open(&filename)?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = pemfile::pkcs8_private_keys(&mut reader)
        .map_err(|_| error("failed to load private key"))?;
    if keys.len() != 1 {
        return Err(error("expected a single private key"));
    }
    Ok(keys[0].clone())
}

fn make_ca_cert<P: AsRef<Path>>(ca_cert_paths: &CertPathPair<P>) -> Try {
    // Example of ca root here:
    // https://github.com/est31/rcgen/blob/a3a3d753fec8ab987fecd13540ecbf251850f43f/tests/webpki.rs#L130
    // TODO Use ca cert for checking later.
    // CA cert.
    let mut ca_params = CertificateParams::default();
    ca_params.distinguished_name = DistinguishedName::new();
    ca_params.distinguished_name.push(DnType::CommonName, "fly ca");
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
    rand_serial(&mut ca_params)?;
    let ca_cert = Certificate::from_params(ca_params)?;
    fs::File::create(&ca_cert_paths.cert_path)?
        .write_all(ca_cert.serialize_pem()?.as_bytes())?;
    write_secret(
        &ca_cert_paths.key_path, ca_cert.serialize_private_key_pem().as_bytes(),
    )?;
    // Prep for checking cert. TODO Probably elsewhere.
    let ca_der = ca_cert.serialize_der()?;
    let trust_anchor_list = &[cert_der_as_trust_anchor(Input::from(&ca_der))?];
    let _trust_anchors = TLSServerTrustAnchors(trust_anchor_list);
    // Done.
    Ok(())
}

fn make_node_cert<P: AsRef<Path>>(
    ca_cert: &CertPair, cert_paths: &CertPathPair<P>,
) -> Try {
    let _ca_cert_bytes = &ca_cert.certs[0].0;
    let _ca_key_bytes = &ca_cert.key.0;
    let ca_key_pair = _ca_key_bytes.as_slice().try_into()?;
    let ca_cert_params = CertificateParams::from_ca_cert_der(
        _ca_cert_bytes.as_slice(), ca_key_pair,
    )?;
    let ca_cert_obj = Certificate::from_params(ca_cert_params)?;
    // Derived cert.
    let mut params = CertificateParams::default();
    params.distinguished_name = DistinguishedName::new();
    // TODO Also, subject alt name, and use actual node/host names?
    params.distinguished_name.push(DnType::CommonName, "fly node");
    rand_serial(&mut params)?;
    let gen_cert = Certificate::from_params(params)?;
    fs::File::create(&cert_paths.cert_path)?.write_all(
        gen_cert.serialize_pem_with_signer(&ca_cert_obj)?.as_bytes(),
    )?;
    write_secret(
        &cert_paths.key_path, gen_cert.serialize_private_key_pem().as_bytes(),
    )?;
    Ok(())
}

fn rand_serial(params: &mut CertificateParams) -> Try {
    let mut bytes: [u8; 8] = [0; 8];
    ring::rand::SystemRandom::new().fill(&mut bytes)?;
    params.serial_number = Some(u64::from_ne_bytes(bytes));
    Ok(())
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
