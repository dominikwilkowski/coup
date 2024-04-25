//! The bot trait [BotInterface] and a couple types that help with the bot implementation

use std::fmt;

use crate::{Action, Card, History, Score};

/// A bot struct can be used to implement the [BotInterface] trait
#[derive(Debug, Clone)]
pub struct Bot {
	pub name: String,
	pub coins: u8,
	pub cards: Vec<Card>,
}

impl Bot {
	pub fn new(name: String) -> Self {
		Self {
			name,
			coins: 2,
			cards: vec![],
		}
	}
}

/// A type to describe other bots still in the game
#[derive(Debug, Clone, PartialEq)]
pub struct OtherBot {
	/// The name of the bot used to identify it in [Action] and [Counter]
	pub name: String,
	/// The amount of coins this bot has
	pub coins: u8,
	/// The amount of [Card] this bot still have
	pub cards: u8,
}

/// The context struct is what is passed into each of the [BotInterface] methods
/// as arguments so the bot knows the context of the current move
#[derive(Debug, Clone)]
pub struct Context {
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
	fn get_coins(&self) -> u8;
	fn set_coins(&mut self, coins: u8);
	fn get_cards(&self) -> Vec<Card>;
	fn set_cards(&mut self, cards: Vec<Card>);

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
		_new_cards: &[Card],
		_context: &Context,
	) -> Option<Vec<Card>> {
		None
	}

	/// Called when you lost a card and now must decide which one you want to lose
	fn on_card_loss(&self, _context: &Context) -> Card {
		self.get_cards().pop().unwrap()
	}
}

/// The debug trait has been implemented to support both format and alternate
/// format which means you can print a bot instance with:
/// ```rust
/// # use coup::{bot::BotInterface, bots::StaticBot};
/// let mut bot = Box::new(StaticBot::new(String::from("My static bot"))) as Box<dyn BotInterface>;
/// println!("{:?}", bot);
/// // Bot { name: "My static bot", coins: 2, cards: [] }
///
/// // or
/// println!("{:#?}", bot);
/// // Bot {
/// //   name: "My static bot"
/// //   coins: 2
/// //   cards: []
/// // }
/// ```
impl fmt::Debug for dyn BotInterface {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			writeln!(f, "Bot {{")?;
			writeln!(f, "  name: {:?}", self.get_name())?;
			writeln!(f, "  coins: {:?}", self.get_coins())?;
			writeln!(f, "  cards: {:?}", self.get_cards())?;
			write!(f, "}}")
		} else {
			write!(
				f,
				"Bot {{ name: {:?}, coins: {:?}, cards: {:?} }}",
				self.get_name(),
				self.get_coins(),
				self.get_cards()
			)
		}
	}
}

/// The display trait has been implemented which means you can print the avatar
/// of a bot instance with:
/// ```rust
/// # use coup::{bot::BotInterface, bots::StaticBot};
/// let mut bot = Box::new(StaticBot::new(String::from("My static bot"))) as Box<dyn BotInterface>;
/// println!("{}", bot);
/// // [My static bot ♡♡ 💰2]
/// ```

impl fmt::Display for dyn BotInterface {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"\x1b[33m[\x1b[1m{}\x1b[0m \x1b[31m{}{}\x1b[33m 💰{}]\x1b[39m",
			self.get_name(),
			"♥".repeat(self.get_cards().len()),
			"♡".repeat(2 - self.get_cards().len()),
			self.get_coins()
		)
	}
}
