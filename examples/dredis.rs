use anyhow::Result;
use std::{io, net::SocketAddr};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tracing::{info, warn};

const BUF_SIZE: usize = 4096;

/// The main entry point for the Dredis server.
///
/// This function sets up a TCP listener, accepts incoming connections,
/// and spawns a new task for each connection to handle Redis-like commands.
///
/// # Returns
///
/// Returns a `Result<()>` which is `Ok(())` if the server runs successfully,
/// or an error if there's a problem setting up the listener or accepting connections.
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    // build a listener
    let addr = "0.0.0.0:63799";
    let listener = TcpListener::bind(addr).await?;
    info!("Dredis: listening on: {}", addr);
    // accept connections and process them serially
    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("Dredis: accepted connection from: {}", raddr);
        tokio::spawn(async move {
            if let Err(e) = process_redis_conn(stream, raddr).await {
                warn!(
                    "Dredis: error processing connection with {}: {:?}",
                    raddr, e
                );
            }
        });
    }
}

/// Processes a Redis-like connection, handling incoming commands and sending responses.
///
/// This function continuously reads from the TCP stream, processes the incoming data
/// as Redis-like commands, and sends appropriate responses. It runs until the connection
/// is closed or an error occurs.
///
/// # Parameters
///
/// * `stream`: A mutable reference to the `TcpStream` representing the client connection.
/// * `raddr`: The `SocketAddr` of the remote client, used for logging purposes.
///
/// # Returns
///
/// Returns `Ok(())` if the connection is processed successfully and closes normally.
/// Returns an `Err` if an I/O error occurs during reading or writing.
async fn process_redis_conn(mut stream: TcpStream, raddr: SocketAddr) -> Result<()> {
    loop {
        stream.readable().await?;
        let mut buf = Vec::with_capacity(BUF_SIZE);
        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                println!("read {} bytes", n);
                info!("read {} bytes", n);
                let line = String::from_utf8_lossy(&buf);
                info!("read line: {:?}", line);
                stream.write_all(b"+OK\r\n").await?;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    warn!("Connection {} closed", raddr);
    Ok(())
}
