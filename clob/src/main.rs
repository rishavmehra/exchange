use std::{collections::HashMap, sync::{Arc, Mutex}};

use actix_web::{delete, get, post, web::{Data, Json}, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    let orderbook = Arc::new(Mutex::new(Orderbook::default()));
    HttpServer::new(move ||
        App::new()
            .app_data(Data::new(orderbook.clone()))
            .service(create_order)
            .service(delete_order)
    )
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

struct Orderbook {
    bids: HashMap<String, Vec<OpenOrder>>,
    asks: HashMap<String, Vec<OpenOrder>>,
    order_id_index: u64
}

#[derive(Clone)]
struct OpenOrder {
    price: u64,
    quanity: u64,
    side: Side,
    user_id: String,
    order_id: String,
    filled_quantity: u64
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Side {
    Buy,
    Sell
}

impl Default for Orderbook {
    fn default() -> Self {
        Self {
            bids: HashMap::new(),
            asks: HashMap::new(),
            order_id_index: 0
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct  CreateOrder {
    price: u64,
    quantity: u64,
    user_id: String,
    side: Side
}


#[derive(Deserialize, Default)]
struct DeleteOrder {
    // user_id: String,
    order_id: String,
}

#[derive( Serialize, Default)]
struct  CreateOrderResponse { 
    order_id: String,
    quantity: u64,
    filled_quantity: u64,
    remaning_quantity: u128
}

#[derive(Serialize, Default)]
struct DeleteOrderResponse{
    success: bool
}

impl Orderbook {
    fn create_order(&mut self, order: CreateOrder) {
        let order_id = self.order_id_index.to_string();
        self.order_id_index += 1;
        match order.side {
            Side::Buy => {
                let open_order = OpenOrder {
                    price: order.price,
                    quanity: order.quantity,
                    side: order.side,
                    user_id: order.user_id,
                    order_id: order_id,
                    filled_quantity: 0
                };
                self.bids.entry(order.price.to_string()).or_default().push(open_order);
            }
            Side::Sell => {
                let open_order = OpenOrder {
                    price: order.price,
                    quanity: order.quantity,
                    side: order.side,
                    user_id: order.user_id,
                    order_id: order_id,
                    filled_quantity: 0
                };
                self.asks.entry(order.price.to_string()).or_default().push(open_order);

            }
        }
    }

    fn delete_order(&mut self, order: DeleteOrder) {
        // delete the bid
        if let Some(price) = self.bids.iter().find_map(|(price, orders)|{
            if orders.iter().any(|o| o.order_id == order.order_id ){
                Some(price.clone())
            }
            else {
                None
            }
        }){
            if let Some(orders) = self.bids.get_mut(&price){
                orders.retain(|o| o.order_id !=order.order_id);
            }
        }

        //delete the ask
        if let Some(price) = self.asks.iter().find_map(|(price, orders)|{
            if orders.iter().any(|o| o.order_id == order.order_id ){
                Some(price.clone())
            }
            else {
                None
            }
        }){
            if let Some(orders) = self.asks.get_mut(&price){
                orders.retain(|o| o.order_id !=order.order_id);
            }
        }
    }

    fn get_depth(&mut self) -> DepthResponse{
        let mut bids = Vec::new();
        let mut asks = Vec::new();
    
        for(price, orders) in self.bids.iter(){
            bids.push(
                Depth{
                    price: price.parse().unwrap(),
                    quantity: orders.iter().map(|o| o.quanity).sum()
                }
            );
        }
        for(price, orders) in self.asks.iter(){
            asks.push(
                Depth{
                    price: price.parse().unwrap(),
                    quantity: orders.iter().map(|o| o.quanity).sum()
                }
            );
        }
        DepthResponse { bids: bids, asks: asks }
    }

}


#[post("/order")]
async fn create_order(orderbook: Data<Arc<Mutex<Orderbook>>>, order: Json<CreateOrder>) -> impl Responder {
    let mut orderbook = orderbook.lock().unwrap();
    let orderbook = orderbook.create_order(order.0);
    HttpResponse::Ok().json(CreateOrderResponse{
        order_id: String::from("google"),
        quantity: 12,
        filled_quantity: 2,
        remaning_quantity: 9
    })
}


#[delete("/order")]
async fn delete_order(orderbook: Data<Arc<Mutex<Orderbook>>>, order: Json<DeleteOrder>) -> impl Responder {
    let mut orderbook = orderbook.lock().unwrap();
    let orderbook = orderbook.delete_order(order.0);
    HttpResponse::Ok().json(DeleteOrderResponse{
        success: true
    })
}

#[derive(Serialize, Deserialize, Debug)]
struct  DepthResponse{
    bids: Vec<Depth>,
    asks: Vec<Depth>
}


#[derive(Serialize, Deserialize, Debug)]
struct  Depth {
    price: u64,
    quantity: u64
}

#[get("/depth")]
async fn get_depth(orderbook: Data<Arc<Mutex<Orderbook>>>) -> impl Responder {
    let mut orderbook = orderbook.lock().unwrap();
    let depth = orderbook.get_depth();
    HttpResponse::Ok().json(depth)
}
