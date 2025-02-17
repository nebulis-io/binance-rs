use crate::util::*;
use crate::model::*;
use crate::client::*;
use crate::errors::*;
use std::collections::BTreeMap;
use serde_json::Value;
use crate::api::API;
use crate::api::Spot;

#[derive(Clone)]
pub struct Market {
    pub client: Client,
    pub recv_window: u64,
}

// Market Data endpoints
#[cfg(feature = "blocking")]
impl Market {
    // Order book at the default depth of 100
    pub fn get_depth<S>(&self, symbol: S) -> Result<OrderBook>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        self.client.get(API::Spot(Spot::Depth), Some(request))
    }

    // Order book at a custom depth. Currently supported values
    // are 5, 10, 20, 50, 100, 500, 1000 and 5000
    pub fn get_custom_depth<S>(&self, symbol: S, depth: u64) -> Result<OrderBook>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("limit".into(), depth.to_string());
        let request = build_request(&parameters);

        self.client.get(API::Spot(Spot::Depth), Some(request))
    }

    // Latest price for ALL symbols.
    pub fn get_all_prices(&self) -> Result<Prices> {
        self.client.get(API::Spot(Spot::Price), None)
    }

    // Latest price for ONE symbol.
    pub fn get_price<S>(&self, symbol: S) -> Result<SymbolPrice>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        self.client.get(API::Spot(Spot::Price), Some(request))
    }

    // Average price for ONE symbol.
    pub fn get_average_price<S>(&self, symbol: S) -> Result<AveragePrice>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        self.client.get(API::Spot(Spot::AvgPrice), Some(request))
    }

    // Symbols order book ticker
    // -> Best price/qty on the order book for ALL symbols.
    pub fn get_all_book_tickers(&self) -> Result<BookTickers> {
        self.client.get(API::Spot(Spot::BookTicker), None)
    }

    // -> Best price/qty on the order book for ONE symbol
    pub fn get_book_ticker<S>(&self, symbol: S) -> Result<Tickers>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        self.client.get(API::Spot(Spot::BookTicker), Some(request))
    }

    // 24hr ticker price change statistics
    pub fn get_24h_price_stats<S>(&self, symbol: S) -> Result<PriceStats>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        self.client.get(API::Spot(Spot::Ticker24hr), Some(request))
    }

    // 24hr ticker price change statistics for all symbols
    pub fn get_all_24h_price_stats(&self) -> Result<Vec<PriceStats>> {
        self.client.get(API::Spot(Spot::Ticker24hr), None)
    }

    // Returns up to 'limit' klines for given symbol and interval ("1m", "5m", ...)
    // https://github.com/binance-exchange/binance-official-api-docs/blob/master/rest-api.md#klinecandlestick-data
    pub fn get_klines<S1, S2, S3, S4, S5>(
        &self, symbol: S1, interval: S2, limit: S3, start_time: S4, end_time: S5,
    ) -> Result<KlineSummaries>
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<Option<u16>>,
        S4: Into<Option<u64>>,
        S5: Into<Option<u64>>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("interval".into(), interval.into());

        // Add three optional parameters
        if let Some(lt) = limit.into() {
            parameters.insert("limit".into(), format!("{}", lt));
        }
        if let Some(st) = start_time.into() {
            parameters.insert("startTime".into(), format!("{}", st));
        }
        if let Some(et) = end_time.into() {
            parameters.insert("endTime".into(), format!("{}", et));
        }

        let request = build_request(&parameters);
        let data: Vec<Vec<Value>> = self.client.get(API::Spot(Spot::Klines), Some(request))?;

        let klines = KlineSummaries::AllKlineSummaries(
            data.iter()
                .map(|row| KlineSummary {
                    open_time: to_i64(&row[0]),
                    open: to_f64(&row[1]),
                    high: to_f64(&row[2]),
                    low: to_f64(&row[3]),
                    close: to_f64(&row[4]),
                    volume: to_f64(&row[5]),
                    close_time: to_i64(&row[6]),
                    quote_asset_volume: to_f64(&row[7]),
                    number_of_trades: to_i64(&row[8]),
                    taker_buy_base_asset_volume: to_f64(&row[9]),
                    taker_buy_quote_asset_volume: to_f64(&row[10]),
                })
                .collect(),
        );
        Ok(klines)
    }
}

// Market Data endpoints
#[cfg(not(feature = "blocking"))]
impl Market {
    // Order book at the default depth of 100
    pub async fn get_depth<S>(&self, symbol: S) -> Result<OrderBook>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        self.client.get(API::Spot(Spot::Depth), Some(request)).await
    }

    // Order book at a custom depth. Currently supported values
    // are 5, 10, 20, 50, 100, 500, 1000 and 5000
    pub async fn get_custom_depth<S>(&self, symbol: S, depth: u64) -> Result<OrderBook>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("limit".into(), depth.to_string());
        let request = build_request(&parameters);

        self.client.get(API::Spot(Spot::Depth), Some(request)).await
    }

    // Latest price for ALL symbols.
    pub async fn get_all_prices(&self) -> Result<Prices> {
        self.client.get(API::Spot(Spot::Price), None).await
    }

    // Latest price for ONE symbol.
    pub async fn get_price<S>(&self, symbol: S) -> Result<SymbolPrice>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        self.client.get(API::Spot(Spot::Price), Some(request)).await
    }

    // Average price for ONE symbol.
    pub async fn get_average_price<S>(&self, symbol: S) -> Result<AveragePrice>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        self.client
            .get(API::Spot(Spot::AvgPrice), Some(request))
            .await
    }

    // Symbols order book ticker
    // -> Best price/qty on the order book for ALL symbols.
    pub async fn get_all_book_tickers(&self) -> Result<BookTickers> {
        self.client.get(API::Spot(Spot::BookTicker), None).await
    }

    // -> Best price/qty on the order book for ONE symbol
    pub async fn get_book_ticker<S>(&self, symbol: S) -> Result<Tickers>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        self.client
            .get(API::Spot(Spot::BookTicker), Some(request))
            .await
    }

    // 24hr ticker price change statistics
    pub async fn get_24h_price_stats<S>(&self, symbol: S) -> Result<PriceStats>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(&parameters);

        self.client
            .get(API::Spot(Spot::Ticker24hr), Some(request))
            .await
    }

    // 24hr ticker price change statistics for all symbols
    pub async fn get_all_24h_price_stats(&self) -> Result<Vec<PriceStats>> {
        self.client.get(API::Spot(Spot::Ticker24hr), None).await
    }

    // Returns up to 'limit' klines for given symbol and interval ("1m", "5m", ...)
    // https://github.com/binance-exchange/binance-official-api-docs/blob/master/rest-api.md#klinecandlestick-data
    pub async fn get_klines<S1, S2, S3, S4, S5>(
        &self, symbol: S1, interval: S2, limit: S3, start_time: S4, end_time: S5,
    ) -> Result<KlineSummaries>
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<Option<u16>>,
        S4: Into<Option<u64>>,
        S5: Into<Option<u64>>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("interval".into(), interval.into());

        // Add three optional parameters
        if let Some(lt) = limit.into() {
            parameters.insert("limit".into(), format!("{}", lt));
        }
        if let Some(st) = start_time.into() {
            parameters.insert("startTime".into(), format!("{}", st));
        }
        if let Some(et) = end_time.into() {
            parameters.insert("endTime".into(), format!("{}", et));
        }

        let request = build_request(&parameters);
        let data: Vec<Vec<Value>> = self
            .client
            .get(API::Spot(Spot::Klines), Some(request))
            .await?;

        let klines = KlineSummaries::AllKlineSummaries(
            data.iter()
                .map(|row| KlineSummary {
                    open_time: to_i64(&row[0]),
                    open: to_f64(&row[1]),
                    high: to_f64(&row[2]),
                    low: to_f64(&row[3]),
                    close: to_f64(&row[4]),
                    volume: to_f64(&row[5]),
                    close_time: to_i64(&row[6]),
                    quote_asset_volume: to_f64(&row[7]),
                    number_of_trades: to_i64(&row[8]),
                    taker_buy_base_asset_volume: to_f64(&row[9]),
                    taker_buy_quote_asset_volume: to_f64(&row[10]),
                })
                .collect(),
        );
        Ok(klines)
    }
}
