//! Every game in the collection, in selector order.

use crate::cart::Cart;

pub mod dodgeball;
pub mod tegenwind;

pub const CARTS: &[Cart] = &[dodgeball::CART, tegenwind::CART];
