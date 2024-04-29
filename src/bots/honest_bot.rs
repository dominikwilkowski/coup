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
			return Action::Assassination(target.name.clone());
		} else if context.cards.contains(&Card::Captain) {
			let target =
				context.other_bots.iter().max_by_key(|bot| bot.coins).unwrap();
			return Action::Stealing(target.name.clone());
		} else if context.cards.contains(&Card::Duke) {
			return Action::Tax;
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
		_action: &Action,
		_by: String,
		_context: &Context,
	) -> bool {
		// TODO
		false
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
