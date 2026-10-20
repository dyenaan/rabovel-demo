use serde::Serialize;
use utoipa::ToSchema;

/// One tradable market, in the frontend `Market` shape. NGX-listed and
/// issuer-listed instruments share this shape and the `mkt_<market>_<ticker>`
/// id format.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MarketResponse {
    pub market_id: String,
    pub symbol: String,
    pub asset_name: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub last_price: String,
    pub bid: Option<String>,
    pub ask: String,
    pub status: String,
    pub price_change_24h: String,
    pub price_change_percent_24h: String,
    pub high_24h: String,
    pub low_24h: String,
    /// Units traded on the Rabovel engine in the last 24h.
    pub volume_24h: String,
    pub origin: crate::InstrumentOrigin,
    pub asset_id: String,
    pub mint_address: Option<String>,
    pub settlement_currency: String,
    pub price_source: String,
    pub price_updated_at: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MarketListResponse {
    pub data: Vec<MarketResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MarketDetailResponse {
    pub data: MarketResponse,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderBookLevelResponse {
    pub price: String,
    pub size: String,
    /// Cumulative size from the best level.
    pub total: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderBookResponse {
    pub market_id: String,
    pub bids: Vec<OrderBookLevelResponse>,
    pub asks: Vec<OrderBookLevelResponse>,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublicTradeResponse {
    pub trade_id: String,
    pub market_id: String,
    pub price: String,
    pub quantity: String,
    /// `BUY` or `SELL`: the side that took liquidity.
    pub side: String,
    pub executed_at: String,
}
