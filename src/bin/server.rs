// src/bin/server.rs
use anyhow::Result;
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, BufReader};

use rust_cli_chat::{
    send_frame, recv_frame, gen_x25519_keypair, pubkey_from_bytes,
    shared_secret_bytes, derive_key_from_shared, aead_from_key,
    encrypt_message, decrypt_message,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting server on 127.0.0.1:9000");
    let listener = TcpListener::bind(("127.0.0.1", 9000)).await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepted from {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("client error: {:?}", e);
            }
        });
    }
}

async fn handle_client(mut stream: tokio::net::TcpStream) -> Result<()> {
    // generate ephemeral keypair
    let (sk, pk) = gen_x25519_keypair();
    let pk_bytes = pk.as_bytes();

    // split stream into read/write halves
    let (mut reader, mut writer) = tokio::io::split(stream);

    // 1) send server pubkey
    send_frame(&mut writer, pk_bytes).await?;

    // 2) recv client pubkey
    let client_pub = recv_frame(&mut reader).await?;
    let client_pk = pubkey_from_bytes(&client_pub)?;

    // derive key
    let shared = shared_secret_bytes(&sk, &client_pk);
    let key = derive_key_from_shared(&shared);
    let aead = aead_from_key(&key);
    let aead_r = aead.clone();

    // reader task: prints incoming decrypted messages
    let mut reader_for_task = reader;
    tokio::spawn(async move {
        loop {
            let frame = match recv_frame(&mut reader_for_task).await {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("read frame err: {:?}", e);
                    break;
                }
            };
            match decrypt_message(&aead_r, &frame) {
                Ok(pt) => {
                    let s = String::from_utf8_lossy(&pt);
                    println!("\n[peer] {}\n> ", s);
                }
                Err(e) => {
                    eprintln!("decrypt err: {:?}", e);
                    break;
                }
            }
        }
    });

    // writer: read stdin lines and send encrypted frames
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
