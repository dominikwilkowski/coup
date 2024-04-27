//! The bot trait [BotInterface] and a couple types that help with the bot implementation

use crate::{Action, Card, History, Score};

/// A bot struct can be used to implement the [BotInterface] trait
#[derive(Debug, Clone, Copy)]
pub struct Bot;

/// A type to describe other bots still in the game
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
/// This is where your game state is stored for you so what your current cards
/// are and how many coins you have.
#[derive(Debug, Clone, PartialEq)]
pub struct Context {
	/// Your cards/influences you still have
	pub cards: Vec<Card>,
	/// Your coins
	pub coins: u8,
	/// A list of all other bots minus the yourself
	pub other_bots: Vec<OtherBot>,
	/// A list of all discarded [Card] so far in the game
	pub discard_pile: Vec<Card>,
	/// A list of each events that have happened in this game so far
	pub history: Vec<History>,
	/// The current score of the game
	pub score: Score,
}

/// The BotInterface trait is what drives your bot.
/// You need to store a couple things for yourself which is what the getter and
/// setter methods are for and then implement each method below that defines
/// the behavior of your bot.
/// The default implementation is a static implementation of a bot like the
/// pre-build [crate::bots::StaticBot].
pub trait BotInterface {
	fn get_name(&self) -> String;

	/// Called when it's your turn to decide what to do
	fn on_turn(&self, _context: &Context) -> Action {
		Action::Income
	}

	/// Called when you have equal to or more than 10 coins and must coup
	/// You can use this method internally as well when you decide to coup on your own
	fn on_auto_coup(&self, context: &Context) -> String {
		let target = context.other_bots.iter().min_by_key(|bot| bot.cards).unwrap();
		target.name.clone()
	}

	/// Called when another bot played an action and everyone gets to decide whether they want to challenge that action
	/// [Action::Assassination], [Action::Swapping], [Action::Stealing] and [Action::Tax]
	fn on_challenge_action_round(
		&self,
		_action: &Action,
		_by: String,
		_context: &Context,
	) -> bool {
		false
	}

	/// Called when someone played something that can be countered with a card you may have
	/// [Action::Assassination], [Action::ForeignAid] and [Action::Stealing]
	fn on_counter(
		&self,
		_action: &Action,
		_by: String,
		_context: &Context,
	) -> Option<bool> {
		None
	}

	/// Called when a bot played a counter. Now everyone gets to decided whether they want to challenge it
	/// [Action::Assassination], [Action::ForeignAid] and [Action::Stealing]
	fn on_challenge_counter_round(
		&self,
		_action: &Action,
		_by: String,
		_context: &Context,
	) -> bool {
		false
	}

	/// Called when you played your ambassador and now need to decide which cards you want to keep
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
