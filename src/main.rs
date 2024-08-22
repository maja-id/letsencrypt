use config::Config;
use futures::prelude::*;
use rustls_acme::{caches::DirCache, AcmeConfig};

#[macro_rules_attribute::apply(smol_macros::main!)]
async fn main() {
    let settings = Config::builder()
        .add_source(config::File::with_name("etc/config.toml"))
        .build()
        .unwrap();

    let tcp_listener = smol::net::TcpListener::bind("[::]:443").await.unwrap();

    let domains = settings.get::<Vec<String>>("domains").unwrap();
    let email = settings.get::<String>("email").unwrap();
    let mut tls_incoming = AcmeConfig::new(domains)
        .contact_push("mailto:".to_owned() + &email)
        .cache(DirCache::new("./acme_cache"))
        .incoming(tcp_listener.incoming(), Vec::new());

    while let Some(tls) = tls_incoming.next().await {
        let mut tls = tls.unwrap();
        smol::spawn(async move {
            tls.write_all(HELLO).await.unwrap();
            tls.close().await.unwrap();
        })
        .detach();
    }
}

const HELLO: &[u8] = br#"HTTP/1.1 200 OK
Content-Length: 11
Content-Type: text/plain; charset=utf-8

Hello Tls!"#;
