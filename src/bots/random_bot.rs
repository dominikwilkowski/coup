//! A random bot implementation for you to use to test your own bot with.

use rand::{seq::SliceRandom, thread_rng};

use crate::{
	bot::{BotInterface, Context, OtherBot},
	Action, Card,
};

/// The random bot will not think about anything but will, just like monkey
/// testing, throw some randomness into your tests with your own bot and helps
/// the engine test its robustness.
pub struct RandomBot;

impl BotInterface for RandomBot {
	fn get_name(&self) -> String {
		String::from("RandomBot")
	}

	fn on_turn(&self, context: &Context) -> Action {
		let mut targets = context.playing_bots.clone();
		targets = targets
			.iter()
			.filter(|bot| bot.name != context.name)
			.cloned()
			.collect::<Vec<OtherBot>>();
		targets.shuffle(&mut thread_rng());

		let mut actions = [
			Action::Assassination(targets[0].name.clone()),
			Action::Coup(targets[0].name.clone()),
			Action::ForeignAid,
			Action::Swapping,
			Action::Income,
			Action::Stealing(targets[0].name.clone()),
			Action::Tax,
		];
		actions.shuffle(&mut thread_rng());
		actions[0].clone()
	}

	fn on_auto_coup(&self, context: &Context) -> String {
		let mut targets = context.playing_bots.clone();
		targets = targets
			.iter()
			.filter(|bot| bot.name != context.name)
			.cloned()
			.collect::<Vec<OtherBot>>();
		targets.shuffle(&mut thread_rng());
		targets[0].name.clone()
	}

	fn on_challenge_action_round(
		&self,
		_action: &Action,
		_by: String,
		_context: &Context,
	) -> bool {
		let mut challange = [true, false];
		challange.shuffle(&mut thread_rng());
		challange[0]
	}

	fn on_counter(
		&self,
		_action: &Action,
		_by: String,
		_context: &Context,
	) -> bool {
		let mut counter = [true, false];
		counter.shuffle(&mut thread_rng());
		counter[0]
	}

	fn on_challenge_counter_round(
		&self,
		_action: &Action,
		_by: String,
		_context: &Context,
	) -> bool {
		let mut challange = [true, false];
		challange.shuffle(&mut thread_rng());
		challange[0]
	}

	fn on_swapping_cards(
		&self,
		new_cards: [Card; 2],
		context: &Context,
	) -> [Card; 2] {
		let mut all_visible_cards = context.cards.clone();
		all_visible_cards.extend(new_cards);
		all_visible_cards.shuffle(&mut thread_rng());

		[all_visible_cards[0], all_visible_cards[1]]
	}

	fn on_card_loss(&self, context: &Context) -> Card {
		let mut cards = context.cards.clone();
		cards.shuffle(&mut thread_rng());
		cards[0]
	}
}
