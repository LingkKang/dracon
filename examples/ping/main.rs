use std::net::{SocketAddr, SocketAddrV4};

use logger::Logger;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let input_socket =
        std::env::args().nth(1).expect("No command argument provided");
    Logger::new().set_prefix(input_socket).init();

    #[rustfmt::skip]
    // The initial pool of sockets for all starting nodes.
    let mut sockets: std::collections::HashSet<SocketAddrV4> =
        std::collections::HashSet::from([
            "172.19.0.2:16", 
            "172.19.0.3:16", 
            "172.19.0.4:16"
            ].map(|socket| socket.parse().unwrap()),
        );

    let mut local_socket: Option<SocketAddrV4> = None;

    let args: Vec<String> = std::env::args().collect();

    // Remove local socket from the sockets pool.
    for arg in args {
        let current_socket = arg.parse::<SocketAddrV4>();
        if current_socket.is_ok() {
            let current_socket = current_socket.unwrap();
            if sockets.contains(&current_socket) {
                sockets.remove(&current_socket);
                local_socket = Some(current_socket);
            }
        }
    }

    let local_socket = local_socket.unwrap_or_else(|| {
        log::error!("No socket address provided");
        std::process::exit(0);
    });

    log::info!("Machine started with socket: {}", local_socket);

    let service = rpc::Service::new(
        SocketAddr::from(local_socket),
        rpc::PingRequest::new("Ping".to_string()),
        rpc::PingResponse::new("Pong".to_string()),
    );
    let listen_service = service.clone();

    let task =
        tokio::spawn(async move { listen_service.handle_request().await });

    let slp = 2;
    log::info!("Sleeping for {} seconds to wait for peers to start", slp);
    tokio::time::sleep(tokio::time::Duration::from_secs(slp)).await;

    for socket in sockets {
        let s = service.clone();
        tokio::spawn(
            async move { s.send_request(SocketAddr::from(socket)).await },
        );
    }

    match tokio::time::timeout(tokio::time::Duration::from_secs(30), task).await
    {
        Ok(Ok(_)) => log::debug!("Task completed"),
        Ok(Err(e)) => log::error!("Task failed due to error: {:?}", e),
        Err(e) => log::info!("Task closed due to timeout: {:?}", e),
    }

    std::process::exit(0);
}
