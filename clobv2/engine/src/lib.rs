use common_types::{Order, OrderBook, PriceLevel, Trade, TradeEvent, TradeID, TradeSide};
use rust_decimal::Decimal;

pub trait ExecutionEngine {
    fn process_order(&mut self, order: Order) -> Vec<TradeEvent>;
}

impl ExecutionEngine for OrderBook {
    fn process_order(&mut self, mut order: Order) -> Vec<TradeEvent> {
        let mut events = Vec::new();

        if order.trade_side == TradeSide::Buy {
            match_buy_order(self, &mut order, &mut events);
        }
        else {
            match_sell_order(self, &mut order, &mut events);
        }

        if order.trade_quantity > Decimal::ZERO{
            place_order(self, order, &mut events);
        }

        events
    }
}

// Bid -> Buy
// Ask -> Sell

pub fn match_buy_order(
    book: &mut OrderBook,
    order: &mut Order,
    events: &mut Vec<TradeEvent>
) {
    while order.trade_quantity > Decimal::ZERO {
        let best_ask_price =  match book.ask.first_key_value() {
            Some((price, _)) => *price,
            None => break
        };

        if best_ask_price > order.price  {
            break;
        }

        let mut best_level = book.ask.first_entry().unwrap().into_mut();

        execute_trade(order, &mut best_level, events, &mut book.next_trade_id);

        if best_level.is_empty() {
            book.ask.remove(&best_ask_price);
        }
    }
}


// Bid -> Buy
// Ask -> Sell

pub fn match_sell_order(
    book: &mut OrderBook,
    order: &mut Order,
    events: &mut Vec<TradeEvent>
) {
    while order.trade_quantity > Decimal::ZERO{
        let best_bid_price = match book.bids.first_key_value() {
            Some((price, _)) => *price,
            None => break
        };

        if best_bid_price < std::cmp::Reverse(order.price) {
            break;
        }

        let best_level = book.ask.first_entry().unwrap().into_mut();
        execute_trade(order, best_level, events, &mut book.next_trade_id);

        if best_level.is_empty() {
            book.bids.remove(&best_bid_price);
        }
    }
}


pub fn place_order(
    book: &mut OrderBook,
    order: Order,
    events: &mut Vec<TradeEvent>
) {
    let price_level = match order.trade_side {
        TradeSide::Buy => book.bids.entry(std::cmp::Reverse(order.price)).or_default(),
        TradeSide::Sell => book.ask.entry(order.price).or_default(),
    };

    price_level.push_back(order);

    events.push(TradeEvent::OrderPlaced { order_id: order.order_id, trader_id: order.trader_id, market_id: order.market_id, side: order.trade_side, price: order.price, quantity: order.trade_quantity, order_type: order.order_type, timestamp: order.timestamp });

}

fn execute_trade(taker_order: &mut Order, maker_price_level:  &mut PriceLevel, events: &mut Vec<TradeEvent>, next_trade_id: &mut u64) {
    let mut filled_maker_orders = Vec::new();
    for (i, maker_order) in maker_price_level.iter_mut().enumerate() {
        // stop if the taker order is completely filled
        if taker_order.trade_quantity == Decimal::ZERO {
            break;
        }

        // calculate how much can we traded 
        // example: If taker 10 and maker  has5, trade_qty  = min(10, 5) = 5
        let trade_qty = std::cmp::min(taker_order.trade_quantity, maker_order.trade_quantity);

        // update quantities
        // exmaple: if trade_qty is 5 
        // taker_order.trade_quantity = 10 -5 = 5 (remaining)
        // maker_order.trade_quantity = 5-5 = 0 (fully filled)
        taker_order.trade_quantity -= trade_qty;
        maker_order.trade_quantity -= trade_qty;

        *next_trade_id += 1;
        events.push(TradeEvent::OrderTraded(Trade {
            trade_id: TradeID(*next_trade_id),
            market_id: taker_order.market_id,
            maker_order_id: maker_order.order_id,
            taker_order_id: taker_order.order_id,
            maker_user_id: maker_order.trader_id,
            taker_user_id: taker_order.trader_id,
            quantity: trade_qty,
            price: maker_order.price,
            timestamp: taker_order.timestamp
        }));

        // if maker order is completely filled - mark it for removal
        if maker_order.trade_quantity == Decimal::ZERO {
            filled_maker_orders.push(i);
        }

    }

    // its very interesting topic (delete the highest index first, lowest index last -> safe, no accidental skips or wrong removals)
    // remove the completely filled maker orders from highest index to lowest
    // we remove in reverse order so indices dnt shift as we remove
    for i in filled_maker_orders.iter().rev() {
        maker_price_level.remove(*i);
    }

}



mod tests {

    use std::collections::BTreeMap;

    use common_types::{OrderType, Price};
    use rust_decimal_macros::dec;

    use super::*;

    fn create_order(
        order_id: u64,
        trader_id: u64,
        market_id: u64,
        trade_side: TradeSide,
        price: Price,
        trade_quantity: Decimal,
        order_type: OrderType,
        timestamp: u64
    ) -> Order {
        Order {
            order_id,
            trader_id,
            market_id,
            trade_side,
            price,
            trade_quantity,
            order_type,
            timestamp
        }
    }

    #[test]
    fn order_with_no_match(){
        let mut book = OrderBook {
            bids: BTreeMap::new(),
            ask: BTreeMap::new(),
            next_trade_id: 0
        };
        
        let order = create_order(1, 1, 1, TradeSide::Buy, dec!(100.0), dec!(10), OrderType::Limit, 0);
        let events = book.process_order(order);

        assert_eq!(book.bids.len(), 1);
        assert_eq!(book.ask.len(), 0);
        assert_eq!(book.bids.get(&std::cmp::Reverse(dec!(100.0))).unwrap().len(), 1 );
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], TradeEvent::OrderPlaced { .. }))
    }

    #[test]
    fn full_order() {
        let mut book = OrderBook{
            bids: BTreeMap::new(),
            ask: BTreeMap::new(),
            next_trade_id: 0
        };
        let maker_order = create_order(1, 1, 1, TradeSide::Sell, dec!(100.0), dec!(10.0), OrderType::Limit, 0);
        book.process_order(maker_order);
        let taker_order = create_order(2, 2, 1, TradeSide::Buy, dec!(100.0), dec!(10.0), OrderType::Limit, 0);
        let event = book.process_order(taker_order);

        assert!(book.bids.is_empty());
        assert!(book.ask.is_empty());
        assert_eq!(event.len(), 1);
        match &event[0] {
            TradeEvent::OrderTraded(trade) =>{
                assert_eq!(trade.maker_order_id, maker_order.trader_id);
                assert_eq!(trade.taker_user_id, taker_order.trader_id)
            }
            _ => panic!("need some trade events")
        }
    }
}