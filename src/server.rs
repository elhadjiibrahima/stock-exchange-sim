// The server module handles all rpc communication functionalities with investor clients and subscriber clients.

use self::stock_exchange::stock_exchange_service_server::StockExchangeService;
use crate::server::stock_exchange::rpc_order_response::{LoginAck, LoginRej};
use crate::server::stock_exchange::RpcOrderResponse;
use crate::types::common::{InvId, SeqNum, SubId};
use crate::types::portal::PortalTask;
use crate::utils::{
    parse_order_request, parse_seqnum, parse_subscribe_request, wrap_cancel_reject, wrap_event,
    wrap_order_ack, wrap_order_reject, wrap_order_response,
};
use crate::{portal::Portal, types::portal::PortalRequest};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use stock_exchange::{
    rpc_order_request, rpc_order_response, RpcOrderRequest, RpcSubscribeRequest,
    RpcSubscribeResponse,
};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Status, Streaming};

type Stream<T> =
    Pin<Box<dyn tokio_stream::Stream<Item = std::result::Result<T, Status>> + Send + 'static>>;

pub mod stock_exchange {
    tonic::include_proto!("stockexchange");
}

pub struct StockExchangeServer {
    portal: Arc<Mutex<Portal>>,
    order_channels: Mutex<HashMap<InvId, mpsc::Sender<RpcOrderResponse>>>,
    market_id_counter: Mutex<SubId>,
    market_channels: Mutex<HashMap<SubId, mpsc::Sender<RpcSubscribeResponse>>>,
}

unsafe impl Send for StockExchangeServer {}
unsafe impl Sync for StockExchangeServer {}

impl StockExchangeServer {
    pub fn new(investor_config: String, stock_config: String) -> Self {
        StockExchangeServer {
            portal: Arc::new(Mutex::new(Portal::new(investor_config, stock_config))),
            order_channels: Mutex::new(HashMap::new()),
            market_id_counter: Mutex::new(0),
            market_channels: Mutex::new(HashMap::new()),
        }
    }

    // dispatch request to portal and process the triggered tasks
    async fn dispatch_request(&self, seqnum: SeqNum, request: PortalRequest) {
        let mut portal = self.portal.lock().await;
        let results = portal.process_request(seqnum, request);
        for res in results {
            self.process_task(res).await;
        }
    }

    // dispatch task to corresponding channels
    async fn process_task(&self, task: PortalTask) {
        match task {
            PortalTask::EventHistory(sub_id, events) => {
                for event in events {
                    self.dispatch_to_market_channel(sub_id, wrap_event(event))
                        .await
                }
            }
            PortalTask::IncrementalEvent(event) => {
                let mut sub_ids: Vec<SubId> = vec![];
                {
                    let channels = self.market_channels.lock().await;
                    sub_ids.extend(channels.keys().cloned());
                }
                for sub_id in sub_ids {
                    self.dispatch_to_market_channel(sub_id, wrap_event(event.clone()))
                        .await
                }
            }
            PortalTask::OrderAck(inv_id, seqnum, order_id) => {
                self.dispatch_to_order_channel(inv_id, wrap_order_ack(seqnum, order_id))
                    .await
            }
            PortalTask::OrderReject(inv_id, seqnum, reason) => {
                self.dispatch_to_order_channel(inv_id, wrap_order_reject(seqnum, reason))
                    .await
            }
            PortalTask::CancelReject(inv_id, seqnum, reason) => {
                self.dispatch_to_order_channel(inv_id, wrap_cancel_reject(seqnum, reason))
                    .await
            }

            PortalTask::OrderResponse(inv_id, r) => {
                self.dispatch_to_order_channel(inv_id, wrap_order_response(r))
                    .await
            }
        }
    }

    // dispatch order response to corresponding order channel
    async fn dispatch_to_order_channel(&self, inv_id: InvId, event: RpcOrderResponse) -> () {
        let channels = self.order_channels.lock().await;
        let tx = channels.get(&inv_id).unwrap();
        tx.send(event).await.unwrap();
    }

    // dispatch market response to corresponding subscriber channel
    async fn dispatch_to_market_channel(&self, sub_id: SubId, event: RpcSubscribeResponse) -> () {
        let channels = self.market_channels.lock().await;
        let tx = channels.get(&sub_id).unwrap();
        tx.send(event).await.unwrap();
    }
}

#[tonic::async_trait]
impl StockExchangeService for StockExchangeServer {
    type SendOrderStream = Stream<RpcOrderResponse>;
    async fn send_order(
        &self,
        request: tonic::Request<Streaming<RpcOrderRequest>>,
    ) -> Result<tonic::Response<Self::SendOrderStream>, tonic::Status> {
        let shared_self = unsafe { Arc::from_raw(self as *const Self) };
        let mut in_stream = request.into_inner();
        let (tx, mut rx) = mpsc::channel::<RpcOrderResponse>(128);
        let (recv_tx, recv_rx) = mpsc::channel::<Result<RpcOrderResponse, Status>>(128);

        // spawn a thread to process order request
        tokio::spawn(async move {
            let mut inv_id = Box::<InvId>::new(0);

            // we require each session to login first
            if let Ok(Some(RpcOrderRequest {
                request: Some(rpc_order_request::Request::Login(login)),
            })) = in_stream.message().await
            {
                let seqnum = login.seqnum.clone();
                let login_success = {
                    let mut portal = shared_self.portal.lock().await;
                    portal.try_login(login.investor_id, &login.password)
                };
                if login_success {
                    // login success
                    println!("[Login] investor_id={}", login.investor_id);
                    *inv_id = login.investor_id;

                    {
                        // add channel to order_channels
                        let mut channels = shared_self.order_channels.lock().await;
                        channels.insert(login.investor_id, tx.clone());
                    }
                    let response = RpcOrderResponse {
                        response: Some(rpc_order_response::Response::LoginAck(LoginAck {
                            seqnum: seqnum,
                        })),
                    };
                    tx.send(response).await.unwrap();
                } else {
                    // login failed
                    let response = RpcOrderResponse {
                        response: Some(rpc_order_response::Response::LoginRej(LoginRej {
                            seqnum: seqnum,
                            reason: "login failed".to_string(),
                        })),
                    };
                    tx.send(response).await.unwrap();
                    return;
                }
            } else {
                // first request is not login
                let response = RpcOrderResponse {
                    response: Some(rpc_order_response::Response::LoginRej(LoginRej {
                        seqnum: 0,
                        reason: "invalid first request".to_string(),
                    })),
                };
                tx.send(response).await.unwrap();
                return;
            }

            // after login, we can process other requests
            while let Ok(Some(event)) = in_stream.message().await {
                println!(
                    "[Order Request] received order request from inv_id={}",
                    inv_id
                );
                let seqnum = parse_seqnum(&event);
                let portal_req = parse_order_request(*inv_id, event);
                shared_self.dispatch_request(seqnum, portal_req).await;
            }
        });

        // spawn a thread to process order response
        tokio::spawn(async move {
            while let Some(r) = rx.recv().await {
                recv_tx.send(Ok(r)).await.unwrap();
            }
        });

        let out_stream = ReceiverStream::new(recv_rx);
        Ok(tonic::Response::new(Box::pin(out_stream)))
    }

    type SubscribeStream = Stream<RpcSubscribeResponse>;

    async fn subscribe(
        &self,
        request: tonic::Request<Streaming<RpcSubscribeRequest>>,
    ) -> Result<tonic::Response<Self::SubscribeStream>, Status> {
        let shared_self = unsafe { Arc::from_raw(self as *const Self) };
        let mut in_stream = request.into_inner();
        let (tx, mut rx) = mpsc::channel::<RpcSubscribeResponse>(128);
        let (recv_tx, recv_rx) = mpsc::channel::<Result<RpcSubscribeResponse, Status>>(128);

        // generate a new sub_id
        let sub_id = {
            let mut counter = shared_self.market_id_counter.lock().await;
            *counter += 1;
            *counter
        };
        {
            // add channel to market_channels
            let mut channels = shared_self.market_channels.lock().await;
            channels.insert(sub_id, tx);
        }
        println!("[Subcribe] received subscribe request");

        // spawn a thread to process subscribe request
        tokio::spawn(async move {
            while let Ok(Some(_event)) = in_stream.message().await {
                let portal_req = parse_subscribe_request(sub_id);
                shared_self.dispatch_request(0, portal_req).await;
            }
        });

        // spawn a thread to process subscribe response
        tokio::spawn(async move {
            while let Some(r) = rx.recv().await {
                recv_tx.send(Ok(r)).await.unwrap();
            }
        });

        let out_stream = ReceiverStream::new(recv_rx);
        Ok(tonic::Response::new(Box::pin(out_stream)))
    }
}
