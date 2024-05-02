//! An honest bot implementation for you to use to test your own bot with.

use crate::{
	bot::{BotInterface, Context},
	Action, Card,
};

/// The honest bot will try to take all actions it should take without being too
/// smart. It will act on it's own cards, counter other bots if they do
/// something that it can counter based on its cards and will never bluff.
pub struct HonestBot;

impl BotInterface for HonestBot {
	/// HonestBot is the name
	fn get_name(&self) -> String {
		String::from("HonestBot")
	}

	/// Acts on cards it has and falls back to [Action::Income].
	/// Never plays [Action::ForeignAid] or [Action::Swapping].
	fn on_turn(&self, context: &Context) -> Action {
		let target = context
			.playing_bots
			.iter()
			.filter(|bot| bot.name != context.name)
			.min_by_key(|bot| bot.cards)
			.unwrap();

		if context.cards.contains(&Card::Assassin) && context.coins >= 3 {
			Action::Assassination(target.name.clone())
		} else if context.cards.contains(&Card::Captain) {
			Action::Stealing(target.name.clone())
		} else if context.cards.contains(&Card::Duke) {
			Action::Tax
		} else {
			Action::Income
		}
	}

	/// Looks for the bot with the least cards
	fn on_auto_coup(&self, context: &Context) -> String {
		let target = context
			.playing_bots
			.iter()
			.filter(|bot| bot.name != context.name)
			.min_by_key(|bot| bot.cards)
			.unwrap();
		target.name.clone()
	}

	/// Challenges only if it can see all three cards associated with the current
	/// action in either the discard pile or its own hand.
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

	/// Counters only if it has the card to counter
	fn on_counter(
		&self,
		action: &Action,
		_by: String,
		context: &Context,
	) -> bool {
		match action {
			Action::Assassination(_) => context.cards.contains(&Card::Contessa),
			Action::ForeignAid => context.cards.contains(&Card::Duke),
			Action::Stealing(_) => {
				context.cards.contains(&Card::Captain)
					|| context.cards.contains(&Card::Ambassador)
			},
			Action::Coup(_) | Action::Swapping | Action::Income | Action::Tax => {
				unreachable!("Can't challenge couping or Income")
			},
		}
	}

	/// Counter-challenges only if it can see all three cards associated with the
	/// current action in either the discard pile or its own hand.
	fn on_challenge_counter_round(
		&self,
		action: &Action,
		_by: String,
		context: &Context,
	) -> bool {
		let mut all_visible_cards = context.cards.clone();
		all_visible_cards.extend(context.discard_pile.clone());

		match action {
			Action::Assassination(_) => {
				all_visible_cards.iter().filter(|card| **card == Card::Contessa).count()
					== 3
			},
			Action::ForeignAid => context.cards.contains(&Card::Duke),
			Action::Stealing(_) => {
				all_visible_cards.iter().filter(|card| **card == Card::Captain).count()
					== 3 && all_visible_cards
					.iter()
					.filter(|card| **card == Card::Ambassador)
					.count() == 3
			},
			Action::Coup(_) | Action::Income | Action::Swapping | Action::Tax => {
				unreachable!("Can't challenge couping or Income")
			},
		}
	}

	/// Swaps duplicate cards
	fn on_swapping_cards(
		&self,
		new_cards: [Card; 2],
		context: &Context,
	) -> [Card; 2] {
		let mut discard_cards = Vec::new();
		if context.cards[0] == context.cards[1] {
			discard_cards.push(context.cards[0])
		} else {
			discard_cards.push(new_cards[0]);
		}
		discard_cards.push(new_cards[1]);

		[discard_cards[0], discard_cards[1]]
	}

	/// Takes the first card to discard
	fn on_card_loss(&self, context: &Context) -> Card {
		context.cards.clone().pop().unwrap()
	}
}
