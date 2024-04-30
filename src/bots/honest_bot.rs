//! A honest bot implementation for you to use to test your own bot with.

use crate::{
	bot::{BotInterface, Context},
	Action, Card,
};

/// The honest bot will try to take all actions it should take without being too
/// smart. It will act on it's own cards, counter other bots if they do
/// something that it can counter based on its cards and will never bluff.
pub struct HonestBot;

impl BotInterface for HonestBot {
	fn get_name(&self) -> String {
		String::from("HonestBot")
	}

	fn on_turn(&self, context: &Context) -> Action {
		if context.cards.contains(&Card::Assassin) && context.coins >= 3 {
			let target =
				context.other_bots.iter().min_by_key(|bot| bot.cards).unwrap();
			Action::Assassination(target.name.clone())
		} else if context.cards.contains(&Card::Captain) {
			let target =
				context.other_bots.iter().max_by_key(|bot| bot.coins).unwrap();
			Action::Stealing(target.name.clone())
		} else if context.cards.contains(&Card::Duke) {
			Action::Tax
		} else {
			Action::Income
		}
	}

	fn on_auto_coup(&self, context: &Context) -> String {
		let target = context.other_bots.iter().min_by_key(|bot| bot.cards).unwrap();
		target.name.clone()
	}

	fn on_challenge_action_round(
		&self,
		action: &Action,
		_by: String,
		context: &Context,
	) -> bool {
		let mut all_visible_cards = context.cards.clone();
		all_visible_cards.extend(context.discard_pile.clone());

		match action {
			Action::Assassination(_) => {
				all_visible_cards.iter().filter(|card| **card == Card::Assassin).count()
					== 3
			},
			Action::Swapping => {
				all_visible_cards
					.iter()
					.filter(|card| **card == Card::Ambassador)
					.count() == 3
			},
			Action::Stealing(_) => {
				all_visible_cards.iter().filter(|card| **card == Card::Captain).count()
					== 3
			},
			Action::Tax => {
				all_visible_cards.iter().filter(|card| **card == Card::Duke).count()
					== 3
			},
			Action::Coup(_) | Action::ForeignAid | Action::Income => {
				unreachable!("Can't challenge couping or Income")
			},
		}
	}

	fn on_counter(
		&self,
		_action: &Action,
		_by: String,
		_context: &Context,
	) -> Option<bool> {
		// TODO
		None
	}

	fn on_challenge_counter_round(
		&self,
		_action: &Action,
		_by: String,
		_context: &Context,
	) -> bool {
		// TODO
		false
	}

	fn on_swapping_cards(
		&self,
		new_cards: [Card; 2],
		_context: &Context,
	) -> [Card; 2] {
		// TODO
		new_cards
	}

	fn on_card_loss(&self, context: &Context) -> Card {
		// TODO
		context.cards.clone().pop().unwrap()
	}
}
