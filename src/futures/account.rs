use std::collections::BTreeMap;

use crate::futures::model::*;
use crate::util::*;
use crate::client::*;
use crate::errors::*;
use crate::api::API;
use crate::api::Futures;

#[derive(Clone)]
pub struct FuturesAccount {
    pub client: Client,
    pub recv_window: u64,
}

struct OrderRequest {
    pub symbol: String,
    pub qty: Option<f64>,
    pub price: Option<f64>,
    pub stop_price: Option<f64>,
    pub order_side: OrderSide,
    pub order_type: OrderType,
    pub time_in_force: Option<TimeInForce>,
    pub close_position: bool,
    pub reduce_only: Option<bool>,
}
#[allow(clippy::all)]
pub enum TimeInForce {
    GTC,
    IOC,
    FOK,
}

impl From<TimeInForce> for String {
    fn from(item: TimeInForce) -> Self {
        match item {
            TimeInForce::GTC => String::from("GTC"),
            TimeInForce::IOC => String::from("IOC"),
            TimeInForce::FOK => String::from("FOK"),
        }
    }
}

#[allow(clippy::all)]
pub enum MarginType {
    Isolated,
    Cross,
}

impl From<MarginType> for String {
    fn from(item: MarginType) -> Self {
        match item {
            MarginType::Isolated => String::from("ISOLATED"),
            MarginType::Cross => String::from("CROSSED"),
        }
    }
}

pub enum OrderType {
    Limit,
    Market,
    StopLossLimit,
    Stop,
    TakeProfit,
    StopMarket,
    TakeProfitMarket,
}

impl From<OrderType> for String {
    fn from(item: OrderType) -> Self {
        match item {
            OrderType::Limit => String::from("LIMIT"),
            OrderType::Market => String::from("MARKET"),
            OrderType::StopLossLimit => String::from("STOP_LOSS_LIMIT"),
            OrderType::Stop => String::from("STOP"),
            OrderType::TakeProfit => String::from("TAKE_PROFIT"),
            OrderType::StopMarket => String::from("STOP_MARKET"),
            OrderType::TakeProfitMarket => String::from("TAKE_PROFIT_MARKET"),
        }
    }
}

pub enum OrderSide {
    Buy,
    Sell,
}

impl From<OrderSide> for String {
    fn from(item: OrderSide) -> Self {
        match item {
            OrderSide::Buy => String::from("BUY"),
            OrderSide::Sell => String::from("SELL"),
        }
    }
}

#[cfg(feature = "blocking")]
impl FuturesAccount {
    // Set leverage
    pub fn set_leverage<S, I>(&self, symbol: S, leverage: I) -> Result<Leverage>
    where
        S: Into<String>,
        I: Into<u64>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("leverage".into(), leverage.into().to_string());

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Leverage), request)
    }

    // Set margin type
    pub fn set_margin_type<S>(&self, symbol: S, margin_type: MarginType) -> Result<Response>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("marginType".into(), margin_type.into());

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::MarginType), request)
    }

    // All current open orders
    pub fn get_all_open_orders(&self, symbol: Option<String>) -> Result<Vec<Order>> {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        if let Some(symbol) = symbol {
            parameters.insert("symbol".into(), symbol.into());
        }
        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .get_signed(API::Futures(Futures::OpenOrders), Some(request))
    }

    // Cancel all open orders
    pub fn cancel_all_open_orders(&self, symbol: Option<String>) -> Result<Response> {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        if let Some(symbol) = symbol {
            parameters.insert("symbol".into(), symbol.into());
        }
        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .delete_signed(API::Futures(Futures::CancelAllOpenOrders), Some(request))
    }

    // Get Balance
    pub fn get_balance(&self) -> Result<Vec<AccountBalance>> {
        let parameters: BTreeMap<String, String> = BTreeMap::new();

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .get_signed(API::Futures(Futures::Balance), Some(request))
    }

    // Get Positions
    pub fn get_positions(&self, symbol: Option<String>) -> Result<Vec<Position>> {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        if let Some(symbol) = symbol {
            parameters.insert("symbol".into(), symbol.into());
        }
        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .get_signed(API::Futures(Futures::PositionsInformation), Some(request))
    }

    /// Place a market buy order
    pub fn market_buy_order<S, F>(
        &self, symbol: S, qty: F, reduce_only: bool,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: None,
            stop_price: None,
            order_side: OrderSide::Buy,
            order_type: OrderType::Market,
            time_in_force: None,
            close_position: false,
            reduce_only: Some(reduce_only),
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
    }

    /// Place a market sell order
    pub fn market_sell_order<S, F>(
        &self, symbol: S, qty: F, reduce_only: bool,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: None,
            stop_price: None,
            order_side: OrderSide::Sell,
            order_type: OrderType::Market,
            time_in_force: None,
            close_position: false,
            reduce_only: Some(reduce_only),
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
    }

    /// Place a take profit buy order
    pub fn take_profit_buy_order<S, F>(
        &self, symbol: S, qty: F, price: f64, stop_price: f64, reduce_only: bool,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: Some(price),
            stop_price: Some(stop_price),
            order_side: OrderSide::Buy,
            order_type: OrderType::TakeProfit,
            time_in_force: None,
            close_position: false,
            reduce_only: Some(reduce_only),
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
    }

    /// Place a take profit self order
    pub fn take_profit_sell_order<S, F>(
        &self, symbol: S, qty: F, price: f64, stop_price: f64, reduce_only: bool,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: Some(price),
            stop_price: Some(stop_price),
            order_side: OrderSide::Sell,
            order_type: OrderType::TakeProfit,
            time_in_force: None,
            close_position: false,
            reduce_only: Some(reduce_only),
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
    }

    /// Place a stop buy order
    pub fn stop_buy_order<S, F>(
        &self, symbol: S, qty: F, price: f64, stop_price: f64,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: Some(price),
            stop_price: Some(stop_price),
            order_side: OrderSide::Buy,
            order_type: OrderType::Stop,
            time_in_force: None,
            close_position: false,
            reduce_only: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
    }

    /// Place a stop self order
    pub fn stop_sell_order<S, F>(
        &self, symbol: S, qty: F, price: f64, stop_price: f64,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: Some(price),
            stop_price: Some(stop_price),
            order_side: OrderSide::Sell,
            order_type: OrderType::Stop,
            time_in_force: None,
            close_position: false,
            reduce_only: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
    }

    /// Place a stop market buy order
    pub fn stop_market_buy_order<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: None,
            stop_price: Some(price),
            order_side: OrderSide::Buy,
            order_type: OrderType::StopMarket,
            time_in_force: None,
            close_position: false,
            reduce_only: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
    }

    /// Place a stop market self order
    pub fn stop_market_sell_order<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: None,
            stop_price: Some(price),
            order_side: OrderSide::Sell,
            order_type: OrderType::StopMarket,
            time_in_force: None,
            close_position: false,
            reduce_only: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
    }

    /// Place a stop loss long position buy order
    pub fn stop_loss_long_position_order<S, F>(&self, symbol: S, price: F) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: None,
            price: None,
            stop_price: Some(price.into()),
            order_side: OrderSide::Sell,
            order_type: OrderType::StopMarket,
            time_in_force: None,
            close_position: true,
            reduce_only: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
    }

    /// Place a stop loss short position order
    pub fn stop_loss_short_position_order<S, F>(&self, symbol: S, price: F) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: None,
            price: None,
            stop_price: Some(price.into()),
            order_side: OrderSide::Buy,
            order_type: OrderType::StopMarket,
            time_in_force: None,
            close_position: true,
            reduce_only: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
    }

    fn build_order(&self, order: OrderRequest) -> BTreeMap<String, String> {
        let mut order_parameters: BTreeMap<String, String> = BTreeMap::new();

        order_parameters.insert("symbol".into(), order.symbol);
        order_parameters.insert("side".into(), order.order_side.into());
        order_parameters.insert("type".into(), order.order_type.into());
        order_parameters.insert("closePosition".into(), order.close_position.to_string());

        if let Some(reduce_only) = order.reduce_only {
            order_parameters.insert("reduceOnly".into(), reduce_only.to_string());
        }

        if let Some(qty) = order.qty {
            order_parameters.insert("quantity".into(), qty.to_string());
        }

        if let Some(stop_price) = order.stop_price {
            order_parameters.insert("stopPrice".into(), stop_price.to_string());
        }

        if let Some(price) = order.price {
            order_parameters.insert("price".into(), price.to_string());
        }
        if let Some(time_in_force) = order.time_in_force {
            order_parameters.insert("timeInForce".into(), time_in_force.into());
        }

        order_parameters
    }
}

#[cfg(not(feature = "blocking"))]
impl FuturesAccount {
    // Set leverage
    pub async fn set_leverage<S, I>(&self, symbol: S, leverage: I) -> Result<Leverage>
    where
        S: Into<String>,
        I: Into<u64>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("leverage".into(), leverage.into().to_string());

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Leverage), request)
            .await
    }

    // Set margin type
    pub async fn set_margin_type<S>(&self, symbol: S, margin_type: MarginType) -> Result<Response>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("marginType".into(), margin_type.into());

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::MarginType), request)
            .await
    }

    // All current open orders
    pub async fn get_all_open_orders(&self, symbol: Option<String>) -> Result<Vec<Order>> {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        if let Some(symbol) = symbol {
            parameters.insert("symbol".into(), symbol.into());
        }
        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .get_signed(API::Futures(Futures::OpenOrders), Some(request))
            .await
    }

    // Cancel all open orders
    pub async fn cancel_all_open_orders(&self, symbol: Option<String>) -> Result<Response> {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        if let Some(symbol) = symbol {
            parameters.insert("symbol".into(), symbol.into());
        }
        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .delete_signed(API::Futures(Futures::CancelAllOpenOrders), Some(request))
            .await
    }

    // Get Balance
    pub async fn get_balance(&self) -> Result<Vec<AccountBalance>> {
        let parameters: BTreeMap<String, String> = BTreeMap::new();

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .get_signed(API::Futures(Futures::Balance), Some(request))
            .await
    }

    // Get Positions
    pub async fn get_positions(&self, symbol: Option<String>) -> Result<Vec<Position>> {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        if let Some(symbol) = symbol {
            parameters.insert("symbol".into(), symbol.into());
        }
        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .get_signed(API::Futures(Futures::PositionsInformation), Some(request))
            .await
    }

    /// Place a market buy order
    pub async fn market_buy_order<S, F>(
        &self, symbol: S, qty: F, reduce_only: bool,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: None,
            stop_price: None,
            order_side: OrderSide::Buy,
            order_type: OrderType::Market,
            time_in_force: None,
            close_position: false,
            reduce_only: Some(reduce_only),
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
            .await
    }

    /// Place a market sell order
    pub async fn market_sell_order<S, F>(
        &self, symbol: S, qty: F, reduce_only: bool,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: None,
            stop_price: None,
            order_side: OrderSide::Sell,
            order_type: OrderType::Market,
            time_in_force: None,
            close_position: false,
            reduce_only: Some(reduce_only),
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
            .await
    }

    /// Place a take profit buy order
    pub async fn take_profit_buy_order<S, F>(
        &self, symbol: S, qty: F, price: f64, stop_price: f64, reduce_only: bool,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: Some(price),
            stop_price: Some(stop_price),
            order_side: OrderSide::Buy,
            order_type: OrderType::TakeProfit,
            time_in_force: None,
            close_position: false,
            reduce_only: Some(reduce_only),
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
            .await
    }

    /// Place a take profit self order
    pub async fn take_profit_sell_order<S, F>(
        &self, symbol: S, qty: F, price: f64, stop_price: f64, reduce_only: bool,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: Some(price),
            stop_price: Some(stop_price),
            order_side: OrderSide::Sell,
            order_type: OrderType::TakeProfit,
            time_in_force: None,
            close_position: false,
            reduce_only: Some(reduce_only),
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
            .await
    }

    /// Place a stop buy order
    pub async fn stop_buy_order<S, F>(
        &self, symbol: S, qty: F, price: f64, stop_price: f64,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: Some(price),
            stop_price: Some(stop_price),
            order_side: OrderSide::Buy,
            order_type: OrderType::Stop,
            time_in_force: None,
            close_position: false,
            reduce_only: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
            .await
    }

    /// Place a stop self order
    pub async fn stop_sell_order<S, F>(
        &self, symbol: S, qty: F, price: f64, stop_price: f64,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: Some(price),
            stop_price: Some(stop_price),
            order_side: OrderSide::Sell,
            order_type: OrderType::Stop,
            time_in_force: None,
            close_position: false,
            reduce_only: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
            .await
    }

    /// Place a stop market buy order
    pub async fn stop_market_buy_order<S, F>(
        &self, symbol: S, qty: F, price: f64,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: None,
            stop_price: Some(price),
            order_side: OrderSide::Buy,
            order_type: OrderType::StopMarket,
            time_in_force: None,
            close_position: false,
            reduce_only: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
            .await
    }

    /// Place a stop market self order
    pub async fn stop_market_sell_order<S, F>(
        &self, symbol: S, qty: F, price: f64,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: Some(qty.into()),
            price: None,
            stop_price: Some(price),
            order_side: OrderSide::Sell,
            order_type: OrderType::StopMarket,
            time_in_force: None,
            close_position: false,
            reduce_only: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
            .await
    }

    /// Place a stop loss long position buy order
    pub async fn stop_loss_long_position_order<S, F>(
        &self, symbol: S, price: F,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: None,
            price: None,
            stop_price: Some(price.into()),
            order_side: OrderSide::Sell,
            order_type: OrderType::StopMarket,
            time_in_force: None,
            close_position: true,
            reduce_only: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
            .await
    }

    /// Place a stop loss short position order
    pub async fn stop_loss_short_position_order<S, F>(
        &self, symbol: S, price: F,
    ) -> Result<PlacedOrder>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            qty: None,
            price: None,
            stop_price: Some(price.into()),
            order_side: OrderSide::Buy,
            order_type: OrderType::StopMarket,
            time_in_force: None,
            close_position: true,
            reduce_only: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Futures(Futures::Order), request)
            .await
    }

    /// Cancel an order
    pub async fn cancel_order<S>(&self, symbol: S, order_id: u64) -> Result<PlacedOrder>
    where
        S: Into<String>,
    {
        let mut parameters = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("orderId".into(), order_id.to_string());

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .delete_signed(API::Futures(Futures::Order), Some(request))
            .await
    }

    /// Cancel multiples order
    pub async fn cancel_orders<S>(&self, symbol: S, order_ids: &[u64]) -> Result<Vec<PlacedOrder>>
    where
        S: Into<String>,
    {
        let mut parameters = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("orderIdList".into(), format!("{:?}", order_ids));

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .delete_signed(API::Futures(Futures::Order), Some(request))
            .await
    }

    fn build_order(&self, order: OrderRequest) -> BTreeMap<String, String> {
        let mut order_parameters: BTreeMap<String, String> = BTreeMap::new();

        order_parameters.insert("symbol".into(), order.symbol);
        order_parameters.insert("side".into(), order.order_side.into());
        order_parameters.insert("type".into(), order.order_type.into());
        order_parameters.insert("closePosition".into(), order.close_position.to_string());

        if let Some(reduce_only) = order.reduce_only {
            order_parameters.insert("reduceOnly".into(), reduce_only.to_string());
        }

        if let Some(qty) = order.qty {
            order_parameters.insert("quantity".into(), qty.to_string());
        }

        if let Some(stop_price) = order.stop_price {
            order_parameters.insert("stopPrice".into(), stop_price.to_string());
        }

        if let Some(price) = order.price {
            order_parameters.insert("price".into(), price.to_string());
        }
        if let Some(time_in_force) = order.time_in_force {
            order_parameters.insert("timeInForce".into(), time_in_force.into());
        }

        order_parameters
    }
}
