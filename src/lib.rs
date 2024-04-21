use rand::{seq::SliceRandom, thread_rng};

pub mod bot;
pub mod static_bot;

use crate::bot::Bot;

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
	Swapping { initiator: String },
	Stealing { initiator: String, target: String },
	ForeignAid { initiator: String },
	Tax { initiator: String },
	Assassination { initiator: String, target: String },
	Income { initiator: String },
	Coup { initiator: String, target: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Challenge {
	Swapping { initiator: String, target: String },
	Stealing { initiator: String, target: String },
	Tax { initiator: String, target: String },
	Assassination { initiator: String, target: String },
	BlockingForeignAid { initiator: String, target: String },
	BlockingAssassination { initiator: String, target: String },
	BlockingStealing { initiator: String, target: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CounterAction {
	BlockingForeignAid { initiator: String, target: String },
	BlockingAssassination { initiator: String, target: String },
	BlockingStealing { initiator: String, target: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum History {
	Action(Action),
	Challenge(Challenge),
	CounterAction(CounterAction),
}

pub struct Coup {
	bots: Vec<Bot>,
	deck: Vec<Card>,
	discard_pile: Vec<Card>,
	history: Vec<History>,
	score: Vec<(String, u64)>,
}

impl Coup {
	pub fn new(bots: Vec<Bot>) -> Self {
		let score = bots.iter().map(|bot| (bot.name.clone(), 0)).collect();

		Self {
			bots,
			deck: vec![],
			discard_pile: vec![],
			history: vec![],
			score,
		}
	}

	pub fn round(mut self) {
		// A fresh deck
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

		// Give all bots their cards
		for bot in &mut self.bots {
			bot.cards.push(deck.pop().unwrap());
			bot.cards.push(deck.pop().unwrap());
		}

		self.deck = deck;

		// Shuffle all bots each round
		self.bots.shuffle(&mut thread_rng());

		// TODO: make sure you only pick 6 at a time for each round
	}

	pub fn _play(mut self) {
		todo!();
	}

	pub fn _looping(mut self) {
		todo!();
	}
}
