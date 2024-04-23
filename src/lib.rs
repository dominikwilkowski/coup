use std::fmt;

use rand::{seq::SliceRandom, thread_rng};

pub mod bot;
pub mod bots;

use crate::bot::BotInterface;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Card {
	Duke,
	Assassin,
	Ambassador,
	Captain,
	Contessa,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
	Swapping,
	Stealing(String),
	ForeignAid,
	Tax,
	Assassination(String),
	Income,
	Coup(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CounterAction {
	BlockingForeignAid,
	BlockingAssassination,
	BlockingStealing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum History {
	ActionSwapping { initiator: String },
	ActionStealing { initiator: String, target: String },
	ActionForeignAid { initiator: String },
	ActionTax { initiator: String },
	ActionAssassination { initiator: String, target: String },
	ActionIncome { initiator: String },
	ActionCoup { initiator: String, target: String },

	ChallengeSwapping { initiator: String, target: String },
	ChallengeStealing { initiator: String, target: String },
	ChallengeTax { initiator: String, target: String },
	ChallengeAssassination { initiator: String, target: String },
	ChallengeBlockingForeignAid { initiator: String, target: String },
	ChallengeBlockingAssassination { initiator: String, target: String },
	ChallengeBlockingStealing { initiator: String, target: String },

	CounterActionBlockingForeignAid { initiator: String, target: String },
	CounterActionBlockingAssassination { initiator: String, target: String },
	CounterActionBlockingStealing { initiator: String, target: String },
}

pub type Score = Vec<(String, u64)>;

pub struct Coup {
	pub bots: Vec<Box<dyn BotInterface>>,
	pub playing_bots: Vec<usize>,
	pub deck: Vec<Card>,
	pub discard_pile: Vec<Card>,
	pub history: Vec<History>,
	pub score: Score,
}

impl Coup {
	/// Start a new Coup game by passing in all your bots here
	pub fn new(bots: Vec<Box<dyn BotInterface>>) -> Self {
		let score = bots.iter().map(|bot| (bot.get_name().clone(), 0)).collect();

		Self {
			bots,
			playing_bots: vec![],
			deck: vec![],
			discard_pile: vec![],
			history: vec![],
			score,
		}
	}

	/// A public method to get a new deck
	pub fn new_deck() -> Vec<Card> {
		let mut deck = vec![
			Card::Assassin,
			Card::Assassin,
			Card::Assassin,
			Card::Ambassador,
			Card::Ambassador,
			Card::Ambassador,
			Card::Captain,
			Card::Captain,
			Card::Captain,
			Card::Contessa,
			Card::Contessa,
			Card::Contessa,
			Card::Duke,
			Card::Duke,
			Card::Duke,
		];
		deck.shuffle(&mut thread_rng());
		deck
	}

	/// Starting a round means we setup the table, give each bots their cards and coins
	pub fn start_round(&mut self) {
		// A fresh deck
		let mut deck = Coup::new_deck();

		// All bots get cards, coins and are referenced in our list of playing bots
		self.playing_bots.clear();
		for (index, bot) in self.bots.iter_mut().enumerate() {
			let new_cards = vec![deck.pop().unwrap(), deck.pop().unwrap()];
			bot.set_cards(new_cards);

			// Add all bots to this round to be later shuffled and truncated
			self.playing_bots.push(index);

			bot.set_coins(2);
		}
		self.deck = deck;

		// Shuffle all bots each round and limit them to the max players per game
		self.playing_bots.shuffle(&mut thread_rng());
		self.playing_bots.truncate(6);

		// TODO: run play() in loop
	}

	/// Play the game with the round that has been setup
	pub fn _play(&mut self) {
		todo!();
	}

	#[allow(clippy::borrowed_box)]
	fn _get_bot_by_name(&self, name: String) -> &Box<dyn BotInterface> {
		self.bots.iter().find(|bot| bot.get_name() == name).unwrap()
	}

	/// Play n number of rounds and tally up the score in the CLI
	pub fn _looping(&mut self) {
		todo!();
	}
}

impl fmt::Debug for Coup {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			writeln!(f, "Coup {{")?;
			writeln!(f, "  bots: {:#?}", self.bots)?;
			writeln!(f, "  playing_bots: {:?}", self.playing_bots)?;
			writeln!(f, "  deck: {:?}", self.deck)?;
			writeln!(f, "  discard_pile: {:?}", self.discard_pile)?;
			writeln!(f, "  history: {:?}", self.history)?;
			writeln!(f, "  score: {:?}", self.score)?;
			write!(f, "}}")
		} else {
			write!(
				f,
				"Coup {{ bots: {:?}, playing_bots: {:?}, deck: {:?}, discard_pile: {:?}, history: {:?}, score: {:?} }}",
				self.bots, self.playing_bots, self.deck, self.discard_pile, self.history, self.score
			)
		}
	}
}
