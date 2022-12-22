/*
* Eiger coding challenge
*/

use std::io;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;

const LOCALHOST: &str = "127.0.0.1";
const WAIT_INTERVAL: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() {
    // Bind port for listening
    let (l, remote_port) = match bind_port().await {
        Ok((l, p)) => (l, p),
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    // Listen and connect to remote in parallel (Exit on Ctrl+C)
    tokio::select! {
        r = listen(l) => { if let Err(e) = r { eprintln!("{e}") } },
        _ = connect(remote_port) => { println!("handshake intiated and completed.") },
        _ = tokio::signal::ctrl_c() => { eprintln!("Ctrl+C exiting") }
    }
}

// Makes a connection to remote/peer. Tries until connection succeeds.
// Calls handshake on a succeeded remote and breaks loop.
async fn connect(remote_port: u16) {
    loop {
        // Make a connection
        if let Ok(s) = TcpStream::connect((LOCALHOST, remote_port)).await {
            // Start communication
            if let Err(e) = do_handshake(s).await {
                eprintln!("{e}");
            }
            break;
        } else {
            println!("Waiting for remote");
            // Wait some time and try again
            sleep(WAIT_INTERVAL).await;
        };
    }
}

// Listens for connections from another peer. Processes incoming connection and breaks loop.
async fn listen(l: TcpListener) -> io::Result<()> {
    loop {
        // addr is not used as we know it is localhost
        let (sock, _addr) = l.accept().await?;
        if let Err(e) = process_incoming(sock).await {
            eprintln!("handshake failed, try again. {e}");
        } else {
            println!("handshake success.");
            break;
        }
    }
    Ok(())
}

// Returns listener and remote port
async fn bind_port() -> io::Result<(TcpListener, u16)> {
    // Bind to port 4000
    match TcpListener::bind((LOCALHOST, 4000)).await {
        Ok(l) => Ok((l, 4001)), // If succeeds then remote is port 4001
        // If fails, then 4000 is already running another instance so use 4001
        Err(_) => TcpListener::bind((LOCALHOST, 4001))
            .await
            .map(|l| (l, 4000)), // In this case remote port is 4000
    }
}

async fn read_line(s: &mut BufReader<TcpStream>) -> io::Result<()> {
    let mut buf = String::new();
    s.read_line(&mut buf).await?;
    print!("{buf}");
    Ok(())
}

// Initiate handshake
async fn do_handshake(s: TcpStream) -> io::Result<()> {
    // Wrap the stream in BufReader so read_line can be used
    let mut s = BufReader::new(s);

    s.write_all(b"1. Hello! Here are my encryption methods.\n")
        .await?;
    read_line(&mut s).await?;
    s.write_all(b"3. Here is encrypted secret-key.\n").await?;
    read_line(&mut s).await?;
    s.write_all(b"5. This is encrypted sample message.\n")
        .await?;
    read_line(&mut s).await
}

// Process incoming handshake request
async fn process_incoming(s: TcpStream) -> io::Result<()> {
    // Wrap the stream in BufReader so read_line can be used
    let mut s = BufReader::new(s);

    read_line(&mut s).await?;
    s.write_all(b"2. Hello! Here is my public key\n").await?;
    read_line(&mut s).await?;
    s.write_all(b"4. Got secret-key.\n").await?;
    read_line(&mut s).await?;
    s.write_all(b"6. Verified sample msg. All OK.\n").await
}
