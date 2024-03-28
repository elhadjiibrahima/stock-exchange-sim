use ses::server::stock_exchange::{
    rpc_subscribe_response::Response, stock_exchange_service_client::StockExchangeServiceClient,
    RpcSubscribeRequest, RpcSubscribeResponse,
};
use tokio_stream::iter;

pub mod subscriber {
    tonic::include_proto!("stockexchange");
}

fn get_subscribe_request() -> Vec<RpcSubscribeRequest> {
    vec![RpcSubscribeRequest {}]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = StockExchangeServiceClient::connect("http://127.0.0.1:50051").await?;
    let mut response_stream = client
        .subscribe(iter(get_subscribe_request()))
        .await?
        .into_inner();

    while let Some(response) = response_stream.message().await? {
        log_response(response);
    }
    Ok(())
}

fn log_response(response: RpcSubscribeResponse) {
    let log = match response.response.unwrap() {
        Response::Added(added) => format!("{:?}", added),
        Response::Removed(removed) => format!("{:?}", removed),
        Response::Executed(executed) => format!("{:?}", executed),
    };
    println!("{}", log);
}
