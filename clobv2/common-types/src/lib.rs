use std::collections::{BTreeMap, VecDeque};

use rust_decimal::Decimal;

#[derive(PartialEq, Copy, Clone)]
pub enum TradeSide {
    Buy,
    Sell
}

#[derive(Copy, Clone)]
pub enum OrderType {
    Limit,
}

pub type PriceLevel = VecDeque<Order>;
pub type Price = Decimal;

pub struct OrderBook {
    pub bids: BTreeMap<std::cmp::Reverse<Price>, PriceLevel>,
    pub ask: BTreeMap<Price, PriceLevel>,
    pub next_trade_id: u64
}

#[derive(Copy, Clone)]
pub struct Order {
    pub order_id: u64,
    pub trader_id: u64, 
    pub market_id: u64,
    pub trade_side: TradeSide,
    pub price: Price,
    pub trade_quantity: Decimal,
    pub order_type: OrderType,
    pub timestamp: u64
}

pub struct  TradeID(pub u64);
pub type Quantity = Decimal;
pub struct Trade {
    pub trade_id: TradeID,
    pub market_id: u64,
    pub maker_order_id: u64,
    pub taker_order_id: u64,
    pub maker_user_id: u64,
    pub taker_user_id: u64,
    pub quantity: Quantity,
    pub price: Price,
    pub timestamp: u64
}

pub enum TradeEvent {
    OrderPlaced {
        order_id: u64,
        trader_id: u64,
        market_id: u64,
        side: TradeSide,
        price: Decimal,
        quantity: Decimal,
        order_type: OrderType,
        timestamp: u64
    },
    OrderTraded(Trade),
    OrderCancelled {
        order_id: u64,
        timestamp: u64
    }
}
