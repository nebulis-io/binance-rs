pub mod account;
pub mod general;
pub mod market;
pub mod model;
pub mod userstream;
pub mod websockets;

fn round(base: f64, precision: i32) -> f64 {
    let multiplier = 10.0_f64.powi(precision);
    (base * multiplier).round() / multiplier
}

fn floor(base: f64, precision: i32) -> f64 {
    let multiplier = 10.0_f64.powi(precision);
    (base * multiplier).floor() / multiplier
}

fn ceil(base: f64, precision: i32) -> f64 {
    let multiplier = 10.0_f64.powi(precision);
    (base * multiplier).ceil() / multiplier
}

impl model::Symbol {
    pub fn round_price(&self, price: f64) -> f64 {
        round(price, self.price_precision as i32)
    }

    pub fn round_quantity(&self, qty: f64) -> f64 {
        round(qty, self.quantity_precision as i32)
    }

    pub fn floor_price(&self, price: f64) -> f64 {
        floor(price, self.price_precision as i32)
    }

    pub fn floor_quantity(&self, qty: f64) -> f64 {
        floor(qty, self.quantity_precision as i32)
    }

    pub fn ceil_price(&self, price: f64) -> f64 {
        ceil(price, self.price_precision as i32)
    }

    pub fn ceil_quantity(&self, qty: f64) -> f64 {
        ceil(qty, self.quantity_precision as i32)
    }
}
