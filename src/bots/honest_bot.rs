//! A honest bot implementation for you to use to test your own bot with.

use crate::{
	bot::{BotInterface, Context},
	Action,
};

/// The honest bot will try to take all actions it should take without being too
/// smart. It will act on it's own cards, counter other bots if they do
/// something that it can counter based on its cards and will never bluff.
pub struct HonestBot;

impl BotInterface for HonestBot {
	fn get_name(&self) -> String {
		String::from("HonestBot")
	}

	// TODO: implement honest bot

	fn on_turn(&self, _context: &Context) -> Action {
		Action::Income
	}
}
