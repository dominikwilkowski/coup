use std::fmt;

use crate::{Action, Card, CounterAction, History, Score};

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

pub type OtherBots = Vec<(Box<dyn BotInterface>, u8)>;

pub trait BotInterface {
	fn get_name(&self) -> String;
	fn get_coins(&self) -> u8;
	fn set_coins(&mut self, coins: u8);
	fn get_cards(&self) -> Vec<Card>;
	fn set_cards(&mut self, cards: Vec<Card>);

	/// Called when it's your turn to decide what to do
	fn on_turn(
		&self,
		_other_bots: OtherBots,
		_discard_pile: Vec<Card>,
		_history: History,
		_score: Score,
	) -> Action {
		Action::Income {
			initiator: self.get_name().clone(),
		}
	}

	/// Called when another bot made an action and everyone gets to decide whether they want to challenge that action
	fn on_challenge_action_round(
		&self,
		_action: Action,
		_other_bots: OtherBots,
		_discard_pile: Vec<Card>,
		_history: History,
		_score: Score,
	) -> bool {
		false
	}

	/// Called when someone does something that can be countered with a card: `Action::Assassination`, `Action::ForeignAid` and `Action::Stealing`
	fn on_counter_action(
		&self,
		_action: Action,
		_other_bots: OtherBots,
		_discard_pile: Vec<Card>,
		_history: History,
		_score: Score,
	) -> Option<CounterAction> {
		None
	}

	/// Called when a bot did a counter action and everyone gets to decided whether they want to challenge that counter action
	fn on_counter_action_round(
		&self,
		_action: Action,
		_counterer: String,
		_other_bots: OtherBots,
		_discard_pile: Vec<Card>,
		_history: History,
		_score: Score,
	) -> bool {
		false
	}

	/// Called when you played your ambassador and now need to decide which cards you want to keep
	fn on_swapping_cards(
		&self,
		_new_cards: Vec<Card>,
		_other_bots: OtherBots,
		_discard_pile: Vec<Card>,
		_history: History,
		_score: Score,
	) -> Option<Vec<Card>> {
		None
	}

	/// Called when you lost a card and now must decide which one you want to lose
	fn on_card_loss(
		&self,
		_other_bots: OtherBots,
		_discard_pile: Vec<Card>,
		_history: History,
		_score: Score,
	) -> Card {
		self.get_cards().pop().unwrap()
	}
}

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