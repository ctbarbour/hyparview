use crate::codec::HyParViewCodec;
use crate::messages::Message;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::time::{self, Duration};
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;

#[derive(Debug)]
struct Listener {
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
}

struct Handler {
    connection: Framed<TcpStream, HyParViewCodec>,
    peer_addr: SocketAddr,
}

const MAX_CONNECTIONS: usize = 1024;

pub async fn run(listener: TcpListener) {
    let mut server = Listener {
        listener: listener,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
    };

    tokio::select! {
        res = server.run() => {
            if let Err(e) = res {
                eprintln!("failed to accept socket; err = {:?}", e);
            }
        }
    }
}

impl Listener {
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Wait for a permit to become available
            //
            // `acquire_owned` returns a permit that is bound to the semaphore.
            // When the permit value is dropped, it is automatically returned
            // to the semaphore.
            //
            // `acquire_owned()` returns `Err` when the semaphore has been
            // closed. We don't ever close the semaphore, so `unwrap()` is safe.
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            // Accept a new socket. This will attempt to perform error handling.
            // The `accept` method internally attempts to recover errors, so an
            // error here is non-recoverable.
            let (stream, peer_addr) = self.accept().await?;

            let mut handler = Handler {
                connection: Framed::new(stream, HyParViewCodec::new()),
                peer_addr: peer_addr,
            };

            // Spawn a new task to process the connections. Tokio tasks are like
            // asynchronous green threads and are executed concurrently.
            tokio::spawn(async move {
                // Process the connection. If an error is encountered, log it.
                if let Err(err) = handler.run().await {
                    eprintln!("connection error; err = {:?}", err);
                }
                // Move the permit into the task and drop it after completion.
                // This returns the permit back to the semaphore.
                drop(permit);
            });
        }
    }

    /// Accept an inbound connection.
    ///
    /// Errors are handled by backing off and retrying. An exponential backoff
    /// strategy is used. After the first failure, the task waits for 1 second.
    /// After the second failure, the task waits for 2 seconds. Each subsequent
    /// failure doubles the wait time. If accepting fails on the 6th try after
    /// waiting for 64 seconds, then this function returns with an error.
    async fn accept(&mut self) -> Result<(TcpStream, SocketAddr), Box<dyn std::error::Error>> {
        let mut backoff = 1;

        loop {
            match self.listener.accept().await {
                Ok((socket, peer_addr)) => return Ok((socket, peer_addr)),
                Err(e) => {
                    if backoff > 64 {
                        return Err(Box::new(e));
                    }
                }
            }

            time::sleep(Duration::from_secs(backoff)).await;

            backoff *= 2;
        }
    }
}

impl Handler {
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        while let Some(message) = self.connection.next().await {
            match message {
                Ok(message) => {
                    self.handle_message(message).await?;
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(())
    }

    async fn handle_message(
        &mut self,
        message: Box<dyn Message + Send>,
    ) -> Result<(), Box<dyn Error>> {
        println!("Received message {:?}", message);
        Ok(())
    }
}
