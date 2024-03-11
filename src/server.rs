use crate::{Config, Connection, PeerState, PeerStateDropGuard};
use crate::peer_service::PeerService;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::time::{self, Duration};
use tracing::{debug, error, info, span, warn, Level};

#[derive(Debug)]
struct Listener {
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
    state_holder: PeerStateDropGuard,
}

#[derive(Debug)]
pub struct Handler {
    state: PeerState,
    connection: Connection,
}

const MAX_CONNECTIONS: usize = 1024;

#[tracing::instrument]
pub async fn run(listener: TcpListener, config: Config) {
    let mut server = Listener {
        listener: listener,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        state_holder: PeerStateDropGuard::new(config),
    };

    tokio::select! {
        res = server.run() => {
            if let Err(e) = res {
                tracing::error!("failed to accept socket; err = {:?}", e);
            }
        }
    }
}

impl Listener {
    #[tracing::instrument]
    pub async fn run(&mut self) -> crate::Result<()> {
        // Clone the Arc outside the scope of Task.
        let mut shuffle_state = self.state_holder.state();
        tokio::spawn(async move {
            loop {
                time::sleep(Duration::from_secs(60)).await; // read the shuffle interval from the config
                info!("Shuffling nodes");
                shuffle_state.do_shuffle().await;
            }
        });

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
            let stream = self.accept().await?;

            let mut handler = Handler {
                state: self.state_holder.state(),
                connection: Connection::new(stream),
            };

            // Spawn a new task to process the connections. Tokio tasks are like
            // asynchronous green threads and are executed concurrently.
            tokio::spawn(async move {
                // Process the connection. If an error is encountered, log it.
                if let Err(err) = handler.run().await {
                    tracing::error!("connection error; err = {:?}", err);
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
    async fn accept(&mut self) -> crate::Result<TcpStream> {
        let mut backoff = 1;

        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
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
    async fn run(&mut self) -> crate::Result<()> {
        let maybe_sender = match self.connection.read_frame().await {
            Ok(Some(message)) => {
                let sender = message.sender();
                message.apply(&mut self.state).await?;
                Some(sender.clone()) // can we avoid the clone?
            }
            Ok(None) => None,
            Err(err) => return Err(err),
        };

        if let Some(sender) = maybe_sender {
            let mut peer_service = PeerService::new(sender, );
            peer_service.handle_connection(self.state.clone());
            self.state.add_active_connection(sender, peer_service);
        }

        Ok(())
    }
}
