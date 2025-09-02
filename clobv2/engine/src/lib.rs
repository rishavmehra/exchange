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

            // todo:  match_sell_order
        }
        events
    }
}

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

        let trade = Trade {
            trade_id: TradeID(*next_trade_id),
            market_id: taker_order.market_id,
            maker_order_id: maker_order.order_id,
            taker_order_id: taker_order.order_id,
            maker_user_id: maker_order.trader_id,
            taker_user_id: taker_order.trader_id,
            quantity: trade_qty,
            price: maker_order.price,
            timestamp: taker_order.timestamp
        };
        *next_trade_id += 1;
        events.push(TradeEvent::OrderTraded(trade));

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

pub fn match_sell_order(

) {
    
}


// test is needed