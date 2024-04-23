use std::fmt;

use rand::{seq::SliceRandom, thread_rng};

pub mod bot;
pub mod bots;

use crate::bot::BotInterface;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Card {
	Ambassador,
	Assassin,
	Captain,
	Contessa,
	Duke,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
	Assassination(String),
	Coup(String),
	ForeignAid,
	Swapping,
	Income,
	Stealing(String),
	Tax,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CounterAction {
	BlockingAssassination,
	BlockingForeignAid,
	BlockingStealing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum History {
	ActionAssassination { initiator: String, target: String },
	ActionCoup { initiator: String, target: String },
	ActionForeignAid { initiator: String },
	ActionSwapping { initiator: String },
	ActionIncome { initiator: String },
	ActionStealing { initiator: String, target: String },

	ChallengeAssassination { initiator: String, target: String },
	ChallengeForeignAid { initiator: String, target: String },
	ChallengeSwapping { initiator: String, target: String },
	ChallengeStealing { initiator: String, target: String },
	ChallengeTax { initiator: String, target: String },

	CounterActionBlockingForeignAid { initiator: String, target: String },
	CounterActionBlockingAssassination { initiator: String, target: String },
	CounterActionBlockingStealingAmbassador { initiator: String, target: String },
	CounterActionBlockingStealingCaptain { initiator: String, target: String },
}

pub type Score = Vec<(String, u64)>;

pub struct Coup {
	pub bots: Vec<Box<dyn BotInterface>>,
	pub playing_bots: Vec<usize>,
	pub deck: Vec<Card>,
	pub discard_pile: Vec<Card>,
	pub history: Vec<History>,
	pub score: Score,
	pub turn: usize,
}

impl Coup {
	/// Start a new Coup game by passing in all your bots in a Vec
	pub fn new(bots: Vec<Box<dyn BotInterface>>) -> Self {
		let score = bots.iter().map(|bot| (bot.get_name().clone(), 0)).collect();

		Self {
			bots,
			playing_bots: vec![],
			deck: vec![],
			discard_pile: vec![],
			history: vec![],
			score,
			turn: 0,
		}
	}

	/// A public method to get a new deck
	pub fn new_deck() -> Vec<Card> {
		let mut deck = vec![
			Card::Ambassador,
			Card::Ambassador,
			Card::Ambassador,
			Card::Assassin,
			Card::Assassin,
			Card::Assassin,
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

	fn _get_score(&self) -> Score {
		self
			.playing_bots
			.iter()
			.map(|bot_index| {
				let bot = &self.bots[*bot_index];
				(bot.get_name().clone(), bot.get_cards().len() as u64)
			})
			.collect()
	}

	fn has_not_ended(&self) -> bool {
		self
			.playing_bots
			.iter()
			.filter(|bot_index| {
				let bot = &self.bots[**bot_index];
				bot.get_coins() > 0
			})
			.count()
			> 1
	}

	/// Starting a round means we setup the table, give each bots their cards and coins
	pub fn play(&mut self) {
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

		// Let's play
		self.round();
	}

	/// Play the game with the round that has been setup
	pub fn round(&mut self) {
		while self.has_not_ended() {
			// TODO
			// Run &self.bots[self.playing_bots[self.turn]].on_turn()
			// match on Actions
			// challenge round through each bot by order of turn
			// if challenge resolve and ask for counter challenge
			// etc etc

			// We move to the next turn
			self.turn = if self.turn == 5 { 0 } else { self.turn + 1 };
		}
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
