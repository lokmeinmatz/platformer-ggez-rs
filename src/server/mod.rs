use async_std::prelude::*;
use async_std::{net, task};
use std::sync::atomic::Ordering;
use crate::networking::packets;


pub fn start() {
    println!("Server starting up...");

    task::block_on(run_server());
}

async fn handle_client(mut stream: net::TcpStream) {
    while !crate::SHOULD_TERMINATE.load(Ordering::Relaxed) {
        let mut header_buf = [0u8; 8];
        if let Ok(rres) = async_std::future::timeout(std::time::Duration::from_millis(10), stream.read_exact(&mut header_buf)).await {
            rres.expect("Failed to receive packet header");
            println!("got new packet");
            match packets::decode_next_packet(&header_buf, &mut stream).await {
                Ok(packet) => {
                    println!("new packet: {:?}", packet);
                },
                Err(e) => eprintln!("Error while decoding packet: {}", e)
            }
        }
    }
}

async fn run_server() -> async_std::io::Result<()> {
    let listener: net::TcpListener = net::TcpListener::bind("127.0.0.1:4321").await?;
    let mut incoming = listener.incoming();
    while !crate::SHOULD_TERMINATE.load(Ordering::Relaxed) {
        if let Ok(Some(Ok(stream))) =
            async_std::future::timeout(std::time::Duration::from_millis(10), incoming.next()).await
        {
            println!("new connection");


            task::spawn(handle_client(stream));
        }

        //println!("server still active");
    }

    println!("Server terminated");

    Ok(())
}
