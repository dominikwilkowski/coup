//! The bot trait [BotInterface] and a couple types that help with the bot
//! implementation

use crate::{Action, Card, History, Score};

/// A bot struct can be used to implement the [BotInterface] trait
#[derive(Debug, Clone, Copy)]
pub struct Bot;

/// A description of other bots current state who are still in the game.
#[derive(Debug, Clone, PartialEq)]
pub struct OtherBot {
	/// The name of the bot used to identify it
	pub name: String,
	/// The amount of coins this bot has
	pub coins: u8,
	/// The amount of [Card] this bot still have
	pub cards: u8,
}

/// The context struct is what is passed into each of the [BotInterface] methods
/// as arguments so the bot knows the context of the current move.
/// This is where your game state is stored including your current cards and
/// coins but also what other bots are still in the game, the discard pile etc.
#[derive(Debug, Clone, PartialEq)]
pub struct Context {
	/// Your name in the game as identifier
	pub name: String,
	/// Your cards/influences you still have
	pub cards: Vec<Card>,
	/// Your coins
	pub coins: u8,
	/// A list of all other bots minus the yourself
	pub playing_bots: Vec<OtherBot>,
	/// A list of all discarded [Card] so far in the game
	pub discard_pile: Vec<Card>,
	/// A list of each event that has happened in this game so far
	pub history: Vec<History>,
	/// The current score of the game
	pub score: Score,
}

/// The BotInterface trait is what drives your bot.
/// Implementing each method below will define your bots behavior.
/// The default implementation is a static implementation of a bot like the
/// pre-build [crate::bots::StaticBot].
pub trait BotInterface {
	fn get_name(&self) -> String;

	/// Called when it's your turn to decide what to do
	fn on_turn(&self, _context: &Context) -> Action {
		Action::Income
	}

	/// Called when you have equal to or more than 10 coins and must coup
	/// You can use this method internally as well when you decide to coup on
	/// your own
	fn on_auto_coup(&self, context: &Context) -> String {
		context
			.playing_bots
			.iter()
			.find(|bot| bot.name != context.name)
			.unwrap()
			.name
			.clone()
	}

	/// Called when another bot played an action and everyone gets to decide
	/// whether they want to challenge that action.
	/// Called for:
	/// - [Action::Assassination]
	/// - [Action::Swapping]
	/// - [Action::Stealing]
	/// - [Action::Tax]
	fn on_challenge_action_round(
		&self,
		_action: &Action,
		_by: String,
		_context: &Context,
	) -> bool {
		false
	}

	/// Called when someone played something that can be countered with a card
	/// you may have.
	/// Called for:
	/// - [Action::Assassination]
	/// - [Action::ForeignAid]
	/// - [Action::Stealing]
	fn on_counter(
		&self,
		_action: &Action,
		_by: String,
		_context: &Context,
	) -> Option<bool> {
		None
	}

	/// Called when a bot played a counter. Now everyone gets to decided whether
	/// they want to challenge that counter card.
	/// Called for:
	/// - [Action::Assassination]
	/// - [Action::ForeignAid]
	/// - [Action::Stealing]
	fn on_challenge_counter_round(
		&self,
		_action: &Action,
		_by: String,
		_context: &Context,
	) -> bool {
		false
	}

	/// Called when you played your ambassador and now need to decide which cards
	/// you want to keep.
	/// Return the cards you don't want anymore. They will be shuffled back into
	/// the deck.
	fn on_swapping_cards(
		&self,
		new_cards: [Card; 2],
		_context: &Context,
	) -> [Card; 2] {
		new_cards
	}

	/// Called when you lost a card and now must decide which one you want to lose
	fn on_card_loss(&self, context: &Context) -> Card {
		context.cards.clone().pop().unwrap()
	}
}
