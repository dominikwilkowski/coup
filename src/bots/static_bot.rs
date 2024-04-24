//! A static bot implementation for you to use to test your own bot with.

use crate::{bot::BotInterface, Card};

/// The static bot only takes [crate::Action::Income] on turns and eventually is forced
/// by the engine to coup another bot. It won't challenge, counter or act on its
/// own cards at all.
pub struct StaticBot {
	pub name: String,
	pub coins: u8,
	pub cards: Vec<Card>,
}

impl StaticBot {
	pub fn new(name: String) -> Self {
		Self {
			name,
			coins: 2,
			cards: vec![],
		}
	}
}

impl BotInterface for StaticBot {
	fn get_name(&self) -> String {
		self.name.clone()
	}

	fn get_coins(&self) -> u8 {
		self.coins
	}

	fn set_coins(&mut self, coins: u8) {
		self.coins = coins;
	}

	fn get_cards(&self) -> Vec<Card> {
		(*self.cards).to_vec()
	}

	fn set_cards(&mut self, cards: Vec<Card>) {
		self.cards = cards;
	}
}
