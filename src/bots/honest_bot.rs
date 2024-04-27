//! A honest bot implementation for you to use to test your own bot with.

use crate::{
	bot::{BotInterface, Context},
	Action,
};

/// The honest bot will try to take all actions it should take without being too
/// smart or strategic thinking. It will act on it's own cards, counter other
/// bots if they do something that it can counter based on its cards and will
/// never bluff itself
pub struct HonestBot;

impl BotInterface for HonestBot {
	fn get_name(&self) -> String {
		String::from("HonestBot")
	}

	fn on_turn(&self, context: &Context) -> Action {
		if context.coins >= 10 {
			let target =
				context.other_bots.iter().min_by_key(|bot| bot.cards).unwrap();
			Action::Coup(target.name.clone())
		} else {
			Action::Income
		}
	}
}
