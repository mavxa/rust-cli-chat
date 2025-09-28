// src/bin/client.rs
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_socks::tcp::Socks5Stream;

use rust_cli_chat::{
    send_frame, recv_frame, gen_x25519_keypair, pubkey_from_bytes,
    shared_secret_bytes, derive_key_from_shared, aead_from_key,
    encrypt_message, decrypt_message,
};

#[tokio::main]
async fn main() -> Result<()> {
    // config: for local test you can use direct TcpStream instead of SOCKS/Tor.
    let use_tor = false;
    let tor_socks = ("127.0.0.1", 9050); // Tor local socks
    let onion_addr = "REPLACE_WITH_ONION_ADDRESS.onion"; // replace for real Tor
    let onion_port = 9000u16;

    // connect
    let stream = if use_tor {
        println!("Connecting via Tor SOCKS5 {}:{}", tor_socks.0, tor_socks.1);
        let sock = Socks5Stream::connect(tor_socks, (onion_addr, onion_port)).await?;
        sock.into_inner()
    } else {
        println!("Connecting to 127.0.0.1:9000 directly");
        tokio::net::TcpStream::connect(("127.0.0.1", 9000)).await?
    };

    let (mut reader, mut writer) = tokio::io::split(stream);

    // generate ephemeral keys
    let (sk, pk) = gen_x25519_keypair();

    // 1) receive server pubkey
    let server_pub = recv_frame(&mut reader).await?;
    let server_pk = pubkey_from_bytes(&server_pub)?;

    // 2) send our pubkey
    send_frame(&mut writer, pk.as_bytes()).await?;

    // derive shared
    let shared = shared_secret_bytes(&sk, &server_pk);
    let key = derive_key_from_shared(&shared);
    let aead = aead_from_key(&key);
    let aead_r = aead.clone();

    // reader task
    let mut reader_for_task = reader;
    tokio::spawn(async move {
        loop {
            match recv_frame(&mut reader_for_task).await {
                Ok(frame) => {
                    match decrypt_message(&aead_r, &frame) {
                        Ok(pt) => println!("\n[peer] {}\n> ", String::from_utf8_lossy(&pt)),
                        Err(e) => { eprintln!("decrypt err: {:?}", e); break; }
                    }
                }
                Err(e) => { eprintln!("read err: {:?}", e); break; }
            }
        }
    });

    // writer: stdin => encrypt => send
    let stdin = tokio::io::stdin();
    let mut stdin_reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        line.clear();
        let n = stdin_reader.read_line(&mut line).await?;
        if n == 0 { break; }
        if line.trim().is_empty() { continue; }
        if line.trim() == "/quit" { break; }
        let payload = encrypt_message(&aead, line.as_bytes());
        send_frame(&mut writer, &payload).await?;
    }

    Ok(())
}
