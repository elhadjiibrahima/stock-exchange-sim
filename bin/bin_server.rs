use ses::server::stock_exchange::stock_exchange_service_server;
use ses::server::StockExchangeServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse().unwrap();
    let mut builder = Server::builder();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        panic!(
            "Usage: {} <cargo run --bin server <investor config file> <stock config file>",
            args[0]
        );
    }

    let investor_config = &args[1];
    let stock_config = &args[2];

    let exchange_core =
        StockExchangeServer::new(investor_config.to_string(), stock_config.to_string());
    let exchange_service =
        stock_exchange_service_server::StockExchangeServiceServer::new(exchange_core);

    builder.add_service(exchange_service).serve(addr).await?;

    Ok(())
}
