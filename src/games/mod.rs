//! Every game in the collection, in selector order.

use crate::cart::Cart;

pub mod dodgeball;
pub mod discdogs;

pub const CARTS: &[Cart] = &[dodgeball::CART, discdogs::CART];
