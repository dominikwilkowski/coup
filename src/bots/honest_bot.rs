//! A honest bot implementation for you to use to test your own bot with.

use crate::{
	bot::{BotInterface, Context},
	Action, Card,
};

/// The honest bot will try to take all actions it should take without being too
/// smart or strategic thinking. It will act on it's own cards, counter other
/// bots if they do something that it can counter based on its cards and will
/// never bluff itself
pub struct HonestBot {
	pub name: String,
	pub coins: u8,
	pub cards: Vec<Card>,
}

impl HonestBot {
	pub fn new(name: String) -> Self {
		Self {
			name,
			coins: 2,
			cards: vec![],
		}
	}
}

impl BotInterface for HonestBot {
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

	fn on_turn(&self, context: Context) -> Action {
		if self.get_coins() >= 10 {
			let target =
				context.other_bots.iter().min_by_key(|bot| bot.cards).unwrap();
			Action::Coup(target.name.clone())
		} else {
			Action::Income
		}
	}
}
