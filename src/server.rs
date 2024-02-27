use crate::codec::HyParViewCodec;
use crate::state::*;
use crate::messages::*;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{self, Duration};
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;
use futures::SinkExt;

#[derive(Debug)]
struct Listener {
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
}

#[derive(Debug)]
pub struct MessageHandler {
    state: Arc<Mutex<PeerState>>,
    sender: SocketAddr,
    stream: TcpStream,
}

impl Handler for MessageHandler {
    async fn on_join(&mut self, _message: &JoinMessage) -> Result<(), std::io::Error> {
        let mut state = self.state.lock().await;
        state.on_join(self.sender, self.stream.clone()).await;

        Ok(())
    }
}

const MAX_CONNECTIONS: usize = 1024;

pub async fn run(listener: TcpListener) {
    let mut server = Listener {
        listener: listener,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
    };

    let state = Arc::new(Mutex::new(PeerState::new(Config::default())));

    tokio::select! {
        res = server.run(state) => {
            if let Err(e) = res {
                eprintln!("failed to accept socket; err = {:?}", e);
            }
        }
    }
}

impl Listener {
    pub async fn run(&mut self, state: Arc<Mutex<PeerState>>) -> Result<(), Box<dyn std::error::Error>> {
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
            let (stream, addr) = self.accept().await?;

            let state = Arc::clone(&state);

            // Spawn a new task to process the connections. Tokio tasks are like
            // asynchronous green threads and are executed concurrently.
            tokio::spawn(async move {
                // Process the connection. If an error is encountered, log it.
                if let Err(err) = Self::process(state, stream, addr).await {
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

    async fn process(
        state: Arc<Mutex<PeerState>>,
        stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<(), Box<dyn Error>> {
        let mut transport = Framed::new(stream, HyParViewCodec::new());
        match transport.next().await {
            Some(Ok(msg)) => {
                let mut state = state.lock().await;
                state.handle_message(addr, stream, msg).await;
            }
            Some(Err(e)) => {
                eprintln!("An error occurred while processing messages for {}; error = {:?}", addr, e);
            }
            _ => ()
        };

        let maybe_active_peer = state.lock().await.get_active_peer(addr);

        if let Some(mut active_peer) = maybe_active_peer {
            loop {
                tokio::select! {
                    Some(msg) = active_peer.rx.recv() => {
                        active_peer.connection.send(msg).await?;
                    }

                    result = transport.next() => match result {
                        Some(Ok(msg)) => {
                            let mut state = state.lock().await;
                            let _ = state.handle_message(addr, stream, msg).await;
                        }
                        Some(Err(e)) => {
                            eprintln!("An error occurred while processing messages for {}; error = {:?}", addr, e);
                        }
                        None => break,
                    }
                }
            }
        };

        transport.send(Box::new(DisconnectMessage { alive: true, respond: false })).await;
        Ok(())
    }
}
