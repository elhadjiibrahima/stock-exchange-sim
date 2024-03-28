// This investor client is to test preset rpc calls to the server, not actual client code
// This client just load an instruction sequence and send it to the server.

use serde::Deserialize;
use ses::{
    server::stock_exchange::{
        rpc_order_request::{Login, NewOrder, Request},
        rpc_order_response::Response,
        stock_exchange_service_client::StockExchangeServiceClient,
        RpcDirection, RpcLimitOrMarket, RpcOrderRequest, RpcOrderResponse, RpcTimeInForce,
    },
    types::common::{Direction, InvId, LimitOrMarket, Password, TimeInForce},
};
use tokio_stream::iter;

pub mod investor {
    tonic::include_proto!("stockexchange");
}

#[derive(Debug, Deserialize)]
struct Instruction {
    ticker: String,
    direction: Direction,
    size: u32,
    price: f32,
    limit_or_market: LimitOrMarket,
    time_in_force: TimeInForce,
}

#[derive(Debug, Deserialize)]
pub struct InvestorTest {
    id: InvId,
    password: Password,
    instructions: Vec<Instruction>,
}

impl InvestorTest {
    pub fn new(file: String) -> Self {
        let contents = std::fs::read_to_string(&file).expect("Unable to read file");
        let investor: InvestorTest = serde_json::from_str(&contents).expect("Failed to parse JSON");
        investor
    }

    // read the instructions from the file and convert them to a list of rpc requests
    pub fn to_request_list(&self) -> Vec<RpcOrderRequest> {
        let mut requests: Vec<RpcOrderRequest> = Vec::new();
        let mut seqnum = 0;

        let login_req = RpcOrderRequest {
            request: Some(Request::Login(Login {
                seqnum,
                investor_id: self.id,
                password: self.password.to_string(),
            })),
        };
        requests.push(login_req);

        for instruction in &self.instructions {
            seqnum += 1;
            let new_order_req: RpcOrderRequest = RpcOrderRequest {
                request: Some(Request::NewOrder(NewOrder {
                    seqnum,
                    ticker: instruction.ticker.to_string(),
                    direction: match instruction.direction {
                        Direction::Buy => RpcDirection::Buy.into(),
                        Direction::Sell => RpcDirection::Sell.into(),
                    },
                    size: instruction.size,
                    price: instruction.price,
                    limit_or_market: match instruction.limit_or_market {
                        LimitOrMarket::Limit => RpcLimitOrMarket::Limit.into(),
                        LimitOrMarket::Market => RpcLimitOrMarket::Market.into(),
                    },
                    time_in_force: match instruction.time_in_force {
                        TimeInForce::Day => RpcTimeInForce::Day.into(),
                        TimeInForce::IOC => RpcTimeInForce::Ioc.into(),
                    },
                })),
            };
            requests.push(new_order_req);
        }
        requests
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --bin investor <instruction_file>");
        std::process::exit(1);
    }

    let investor_test = InvestorTest::new(args[1].to_string());

    let mut client = StockExchangeServiceClient::connect("http://127.0.0.1:50051").await?;
    let mut response_stream = client
        .send_order(iter(investor_test.to_request_list()))
        .await?
        .into_inner();

    while let Some(response) = response_stream.message().await? {
        log_response(response);
    }
    Ok(())
}

fn log_response(response: RpcOrderResponse) {
    let log = match response.response.unwrap() {
        Response::LoginAck(login_ack) => format!("{:?}", login_ack),
        Response::LoginRej(login_rej) => format!("{:?}", login_rej),
        Response::Ack(ack) => format!("{:?}", ack),
        Response::Rej(rej) => format!("{:?}", rej),
        Response::Fill(fill) => format!("{:?}", fill),
        Response::Dead(dead) => format!("{:?}", dead),
        Response::CancelRej(cancel_rej) => format!("{:?}", cancel_rej),
    };
    println!("{}", log);
}
