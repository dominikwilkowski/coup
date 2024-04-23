use std::fmt;

use rand::{seq::SliceRandom, thread_rng};

pub mod bot;
pub mod bots;

use crate::bot::{BotInterface, OtherBot};

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

	// fn _get_score(&self) -> Score {
	// 	self
	// 		.playing_bots
	// 		.iter()
	// 		.map(|bot_index| {
	// 			let bot = &self.bots[*bot_index];
	// 			(bot.get_name().clone(), bot.get_cards().len() as u64)
	// 		})
	// 		.collect()
	// }

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

	/// Starting a round which means we setup the table, give each bots their cards and coins
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
		self.game_loop();
	}

	fn get_other_bots(&self) -> Vec<OtherBot> {
		self
			.playing_bots
			.iter()
			.map(|bot_index| {
				let bot = &self.bots[*bot_index];
				OtherBot {
					name: bot.get_name(),
					coins: bot.get_coins(),
					cards: bot.get_cards().len() as u8,
				}
			})
			.filter(|bot| {
				bot.name != self.bots[self.playing_bots[self.turn]].get_name()
			})
			.collect()
	}

	/// Play the game with the round that has been setup
	pub fn game_loop(&mut self) {
		let playing_bot = &self.bots[self.playing_bots[self.turn]];
		let playing_bot_name = playing_bot.get_name();
		let playing_bot_avatar = format!("{}", playing_bot);
		let playing_bot_coins = playing_bot.get_coins();
		let _playing_bot_cards = playing_bot.get_cards();

		let other_bots = self.get_other_bots();
		while self.has_not_ended() {
			let action = &self.bots[self.playing_bots[self.turn]].on_turn(
				&other_bots,
				&self.discard_pile,
				&self.history,
				&self.score,
			);

			// TODO: check if action is legal

			match action {
				Action::Assassination(_target) => {
					todo!()
				},
				Action::Coup(target) => {
					if playing_bot_coins < 7 {
						// TODO: penalty
					} else {
						// Paying the fee
						self.bots[self.playing_bots[self.turn]]
							.set_coins(playing_bot_coins - 7);

						// Taking a card from the target bot
						let target_bot = self.get_bot_by_name(target.clone());
						let target_bot_name = target_bot.get_name();
						let target_bot_avatar = format!("{}", target_bot);

						let lost_card = target_bot.on_card_loss(
							&other_bots,
							&self.discard_pile,
							&self.history,
							&self.score,
						);
						if !target_bot.get_cards().contains(&lost_card) {
							// TODO: penalty!
						} else {
							self.bots.iter_mut().for_each(|bot| {
								if bot.get_name() == target.clone() {
									bot.set_cards(
										bot
											.get_cards()
											.into_iter()
											.filter(|card| lost_card != *card)
											.collect(),
									);
								}
							})
						}

						self.history.push(History::ActionCoup {
							initiator: playing_bot_name.clone(),
							target: target_bot_name,
						});
						println!("🃏  {} coups {}", playing_bot_avatar, target_bot_avatar);
					}
				},
				Action::ForeignAid => {
					todo!()
				},
				Action::Swapping => {
					todo!()
				},
				Action::Income => {
					// Adding the coin to the bot
					self.bots[self.playing_bots[self.turn]]
						.set_coins(playing_bot_coins + 1);

					// Logging
					self.history.push(History::ActionIncome {
						initiator: playing_bot_name.clone(),
					});
					println!("🃏  {} takes \x1b[33ma coin\x1b[39m", playing_bot_avatar);
				},
				Action::Stealing(_target) => {
					todo!()
				},
				Action::Tax => {
					todo!()
				},
			}

			// TODO
			// Run &self.bots[self.playing_bots[self.turn]].on_turn()
			// match on Actions
			// challenge round through each bot by order of turn
			// if challenge resolve and ask for counter challenge
			// etc etc

			// We move to the next turn
			self.turn = if self.turn == 5 { 0 } else { self.turn + 1 };
			break;
		}
	}

	#[allow(clippy::borrowed_box)]
	fn get_bot_by_name(&self, name: String) -> &Box<dyn BotInterface> {
		// TODO: return Option plus add penalty for bots who pass invalid names
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
