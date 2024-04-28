//! A static bot implementation for you to use to test your own bot with.

use crate::bot::BotInterface;

/// The static bot only takes [crate::Action::Income] on turns and is eventually
/// forced by the engine to coup another bot.
/// It won't challenge, counter or act on its own cards at all.
pub struct StaticBot;

impl BotInterface for StaticBot {
	fn get_name(&self) -> String {
		String::from("StaticBot")
	}
}
