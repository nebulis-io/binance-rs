use crate::errors::*;
use crate::config::*;
use crate::futures::model::*;
use futures_util::StreamExt;
use url::Url;
use serde_json::from_str;
use serde::{Deserialize, Serialize};

use std::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
};
use tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tungstenite::handshake::client::Response;
use tokio::net::TcpStream;

static WEBSOCKET_URL: &str = "wss://fstream.binance.com/ws/";
static WEBSOCKET_MULTI_STREAM: &str = "wss://fstream.binance.com/stream?streams="; // <streamName1>/<streamName2>/<streamName3>

static ORDER_TRADE_UPDATE: &str = "ORDER_TRADE_UPDATE";
static ACCOUNT_UPDATE: &str = "ACCOUNT_UPDATE";
static ACCOUNT_CONFIG_UPDATE: &str = "ACCOUNT_CONFIG_UPDATE";
static LISTEN_KEY_EXPIRED: &str = "listenKeyExpired";
static AGGREGATED_TRADE: &str = "aggTrade";
static STREAM: &str = "stream";

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FuturesWebsocketEvent {
    OrderTrade(OrderTradeUpdateEvent),
    AccountUpdate(AccountUpdateEvent),
    LeverageUpdate(LeverageUpdateEvent),
    ListenKeyExpired(ListenKeyExpiredEvent),
    AggTrade(StreamAggTrade)
}

// Account

pub struct FuturesWebSockets<'a, Fut, S>
where
    Fut: Future<Output = Result<()>>,
    S: Send + Sync + Clone,
{
    pub socket: Option<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response)>,
    handler: Box<dyn Fn(FuturesWebsocketEvent, S) -> Fut + 'static + Send + Sync>,
    subscription: &'a str,
    state: S,
}

impl<'a, F, S> FuturesWebSockets<'a, F, S>
where
    F: Future<Output = Result<()>>,
    S: Send + Sync + Clone,
{
    pub fn new<Callback>(handler: Callback, state: S) -> FuturesWebSockets<'a, F, S>
    where
        Callback: Fn(FuturesWebsocketEvent, S) -> F + 'static + Send + Sync,
    {
        FuturesWebSockets {
            socket: None,
            handler: Box::new(handler),
            subscription: "",
            state,
        }
    }

    pub async fn connect(&mut self, subscription: &'a str) -> Result<()> {
        self.subscription = subscription;
        let wss: String = format!("{}{}", WEBSOCKET_URL, subscription);
        let url = Url::parse(&wss)?;

        match connect_async(url).await {
            Ok(answer) => {
                self.socket = Some(answer);
                Ok(())
            }
            Err(e) => {
                bail!(format!("Error during handshake {}", e));
            }
        }
    }

    pub async fn connect_multiple_streams<Str: AsRef<str>>(&mut self, streams: &[Str]) -> Result<()> {
        let wss: String = format!("{}{}", WEBSOCKET_MULTI_STREAM, streams.iter().map(|s| s.as_ref()).collect::<Vec<_>>().join("/"));
        let url = Url::parse(&wss)?;

        match connect_async(url).await {
            Ok(answer) => {
                self.socket = Some(answer);
                Ok(())
            }
            Err(e) => {
                bail!(format!("Error during handshake {}", e));
            }
        }
    }

    pub async fn connect_with_config(
        &mut self, subscription: &'a str, config: &'a Config,
    ) -> Result<()> {
        self.subscription = subscription;
        let wss: String = format!("{}{}", &config.ws_endpoint, subscription);
        let url = Url::parse(&wss)?;

        match connect_async(url).await {
            Ok(answer) => {
                self.socket = Some(answer);
                Ok(())
            }
            Err(e) => {
                bail!(format!("Error during handshake {}", e));
            }
        }
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(ref mut socket) = self.socket {
            Ok(socket.0.close(None).await?)
        } else {
            bail!("Not able to close the connection");
        }
    }

    async fn handle_msg(&mut self, msg: &str) -> Result<()> {
        let value: serde_json::Value = serde_json::from_str(msg)?;
        let msg = if msg.find(STREAM) != None {
            if value["data"] != serde_json::Value::Null {
                format!("{}", value["data"])
            } else {
                msg.to_owned()
            }
        } else {
            msg.to_owned()
        };
        
        if msg.find(ORDER_TRADE_UPDATE) != None {
            let order_trade: OrderTradeUpdateEvent = from_str(&msg)?;
            (self.handler)(
                FuturesWebsocketEvent::OrderTrade(order_trade),
                self.state.clone(),
            )
            .await?;
        } else if msg.find(ACCOUNT_UPDATE) != None {
            let account_update: AccountUpdateEvent = from_str(&msg)?;
            (self.handler)(
                FuturesWebsocketEvent::AccountUpdate(account_update),
                self.state.clone(),
            )
            .await?;
        } else if msg.find(ACCOUNT_CONFIG_UPDATE) != None {
            let leverage_update: LeverageUpdateEvent = from_str(&msg)?;
            (self.handler)(
                FuturesWebsocketEvent::LeverageUpdate(leverage_update),
                self.state.clone(),
            )
            .await?;
        } else if msg.find(LISTEN_KEY_EXPIRED) != None {
            let listen_key_expired: ListenKeyExpiredEvent = from_str(&msg)?;
            (self.handler)(
                FuturesWebsocketEvent::ListenKeyExpired(listen_key_expired),
                self.state.clone(),
            )
            .await?;
        } else if msg.find(AGGREGATED_TRADE) != None {
            let stream_agg_trade: StreamAggTrade = from_str(&msg)?;
            (self.handler)(
                FuturesWebsocketEvent::AggTrade(stream_agg_trade),
                self.state.clone(),
            )
            .await?;
        } else {
            bail!(format!("Can't decode: {:?}", &msg));
        }
        Ok(())
    }

    pub async fn event_loop(&mut self, running: &AtomicBool) -> Result<()> {
        while running.load(Ordering::Relaxed) {
            if let Some(ref mut socket) = self.socket {
                let message = if let Some(message) = socket.0.next().await {
                    message
                } else {
                    continue;
                }?;
                match message {
                    Message::Text(msg) => match self.handle_msg(&msg).await {
                        Ok(_) => {}
                        Err(Error(ErrorKind::ListenKeyExpired, _)) => {
                            bail!(ErrorKind::ListenKeyExpired);
                        }
                        Err(e) => {
                            bail!(format!("Error on handling stream message: {}", e));
                        }
                    },
                    Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => (),
                    Message::Close(e) => {
                        bail!(format!("Disconnected {:?}", e));
                    }
                }
            }
        }
        Ok(())
    }
}
