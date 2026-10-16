//! Every game in the collection, in selector order.

use crate::cart::Cart;

pub mod discdogs;
pub mod dodgeball;
pub mod trailmix;

pub const CARTS: &[Cart] = &[dodgeball::CART, discdogs::CART, trailmix::CART];
