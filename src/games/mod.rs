//! Every game in the collection, in selector order.

use crate::cart::Cart;

pub mod discdogs;
pub mod dodgeball;
pub mod hangtime;
pub mod tegenwind;
pub mod tienen;
pub mod trailmix;
pub mod zenuwen;

pub const CARTS: &[Cart] = &[dodgeball::CART, discdogs::CART, zenuwen::CART, tienen::CART, tegenwind::CART, trailmix::CART, hangtime::CART];
