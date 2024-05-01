//! # Coup
//! This is an engine for the popular card game [coup](http://gamegrumps.wikia.com/wiki/Coup).
//!
//! ```rust
//! use coup::{
//!     bots::{HonestBot, RandomBot, StaticBot},
//!     Coup,
//! };
//!
//! let mut coup_game = Coup::new(vec![
//!     Box::new(StaticBot),
//!     Box::new(HonestBot),
//!     Box::new(RandomBot),
//!     Box::new(StaticBot),
//!     Box::new(RandomBot),
//!     Box::new(HonestBot),
//! ]);
//!
//! // You can play a single game
//! coup_game.play();
//!
//! // Or you can play 5 games (or more)
//! coup_game.looping(5);
//! ```

extern crate cfonts;

use cfonts::{render, Colors, Options};
use rand::{seq::SliceRandom, thread_rng};
use std::fmt;

pub mod bot;
pub mod bots;

use crate::bot::{BotInterface, Context, OtherBot};

/// One of the five cards you get in the game of Coup
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Card {
	/// - [Action::Swapping] – Draw two character cards from the deck, choose which (if any) to exchange with your cards, then return two<br>
	/// - [Counter::Stealing] – Block someone from stealing coins from you
	Ambassador,
	/// - [Action::Assassination] – Pay three coins and try to assassinate another player's character
	Assassin,
	/// - [Action::Stealing] – Take two coins from another player
	/// - [Counter::Stealing] – Block someone from stealing coins from you
	Captain,
	/// - [Counter::Assassination] – Block an assassination attempt against yourself
	Contessa,
	/// - [Action::Tax] – Take three coins from the treasury<br>
	/// - [Counter::ForeignAid] – Block someone from taking foreign aid
	Duke,
}

/// Actions that can we taken with a [Card] you have
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
	/// Take this action with your [Card::Assassin]
	Assassination(String),
	/// This standard action can be taken at any time as long as you have at least
	/// 7 coin
	Coup(String),
	/// This standard action can be taken at any time
	ForeignAid,
	/// Take this action with your [Card::Ambassador]
	Swapping,
	/// This standard action can be taken at any time
	Income,
	/// Take this action with your [Card::Captain]
	Stealing(String),
	/// Take this action with your [Card::Duke]
	Tax,
}

/// Counters are played if something happens that can be countered with a
/// [Card] you have
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Counter {
	/// Block an assassination with your [Card::Contessa]
	Assassination,
	/// Block foreign aid with your [Card::Duke]
	ForeignAid,
	/// Block stealing with your [Card::Captain] or your [Card::Ambassador]
	Stealing,
}

enum ChallengeRound {
	Action,
	Counter,
}

/// A collection on all possible moves in the game for bots to analyze
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum History {
	ActionAssassination { by: String, target: String },
	ActionCoup { by: String, target: String },
	ActionForeignAid { by: String },
	ActionSwapping { by: String },
	ActionIncome { by: String },
	ActionStealing { by: String, target: String },
	ActionTax { by: String },

	ChallengeAssassin { by: String, target: String },
	ChallengeAmbassador { by: String, target: String },
	ChallengeCaptain { by: String, target: String },
	ChallengeDuke { by: String, target: String },

	CounterAssassination { by: String, target: String },
	CounterForeignAid { by: String, target: String },
	CounterStealing { by: String, target: String },

	CounterChallengeContessa { by: String, target: String },
	CounterChallengeDuke { by: String, target: String },
	CounterChallengeCaptainAmbassedor { by: String, target: String },
}

/// The score of the game for all bots
pub type Score = Vec<(String, f64)>;

struct Bot {
	name: String,
	coins: u8,
	cards: Vec<Card>,
	interface: Box<dyn BotInterface>,
}

impl fmt::Debug for Bot {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			writeln!(f, "Bot {{")?;
			writeln!(f, "  name: {:?}", self.name)?;
			writeln!(f, "  coins: {:?}", self.coins)?;
			writeln!(f, "  cards: {:?}", self.cards)?;
			write!(f, "}}")
		} else {
			write!(
				f,
				"Bot {{ name: {:?}, coins: {:?}, cards: {:?} }}",
				self.name, self.coins, self.cards
			)
		}
	}
}

impl fmt::Display for Bot {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"\x1b[33m[\x1b[1m{}\x1b[0m \x1b[31m{}{}\x1b[33m 💰{}]\x1b[39m",
			self.name,
			"♥".repeat(self.cards.len()),
			"♡".repeat(2 - self.cards.len()),
			self.coins
		)
	}
}

/// The Coup game engine
pub struct Coup {
	bots: Vec<Bot>,
	playing_bots: Vec<usize>,
	deck: Vec<Card>,
	discard_pile: Vec<Card>,
	history: Vec<History>,
	score: Score,
	turn: usize,
	moves: usize,
	log: bool,
	rounds: u64,
	round: u64,
}

impl Coup {
	/// Start a new Coup game by passing in all your bots in a Vec
	pub fn new(user_bots: Vec<Box<dyn BotInterface>>) -> Self {
		let mut bots: Vec<Bot> = Vec::new();
		let mut existing_names: Vec<String> = Vec::new();
		let mut score: Vec<(String, f64)> = Vec::new();

		for bot in user_bots.into_iter() {
			let base_name = bot.get_name();

			// Generating a unique name for the bot
			let mut unique_name = base_name.clone();
			let mut suffix = 2;
			while existing_names.contains(&unique_name) {
				unique_name = format!("{} {}", base_name, suffix);
				suffix += 1;
			}

			existing_names.push(unique_name.clone());

			let bot = Bot {
				name: unique_name.clone(),
				coins: 2,
				cards: Vec::new(),
				interface: bot,
			};

			bots.push(bot);
			score.push((unique_name, 0.0));
		}

		Self {
			bots,
			playing_bots: vec![],
			deck: vec![],
			discard_pile: vec![],
			history: vec![],
			score,
			turn: 0,
			moves: 0,
			log: true,
			round: 0,
			rounds: 0,
		}
	}

	/// A public method to get a new deck.
	/// This can be used by bots to make sure you get the same amount of cards as
	/// the engine does
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

	fn setup(&mut self) {
		// A fresh deck
		let mut deck = Coup::new_deck();

		// Put the index of all bots into play so we can shuffle them later
		self.playing_bots.clear();
		for index in 0..self.bots.len() {
			self.playing_bots.push(index);
		}

		// Shuffle all bots each round and limit them to the max players per game
		self.playing_bots.shuffle(&mut thread_rng());
		self.playing_bots.truncate(6);

		// Give all playing bots cards and coins
		for bot in self.playing_bots.iter() {
			let new_cards = vec![deck.pop().unwrap(), deck.pop().unwrap()];
			self.bots[*bot].cards = new_cards;
			self.bots[*bot].coins = 2;
		}
		self.deck = deck;

		self.discard_pile = vec![];
		self.history = vec![];
		self.turn = 0;
		self.moves = 0;
	}

	fn log(message: std::fmt::Arguments, logging: bool) {
		if logging {
			println!(" {:?}", message);
		}
	}

	fn get_bot_by_name(&self, name: String) -> &Bot {
		self.bots.iter().find(|bot| bot.name == name).unwrap()
	}

	fn get_other_bots(&self) -> Vec<OtherBot> {
		self
			.playing_bots
			.iter()
			.map(|bot_index| {
				let bot = &self.bots[*bot_index];
				OtherBot {
					name: bot.name.clone(),
					coins: bot.coins,
					cards: bot.cards.len() as u8,
				}
			})
			.filter(|bot| bot.cards != 0)
			.collect()
	}

	fn get_context(&self, name: String) -> Context {
		Context {
			name: name.clone(),
			coins: self.get_bot_by_name(name.clone()).coins,
			cards: self.get_bot_by_name(name.clone()).cards.clone(),
			playing_bots: self.get_other_bots(),
			discard_pile: self.discard_pile.clone(),
			history: self.history.clone(),
			score: self.score.clone(),
		}
	}

	fn card_loss(&mut self, name: String) {
		if self.get_bot_by_name(name.clone()).cards.is_empty() {
			// This bot is already dead
			return;
		}
		let context = self.get_context(name.clone());
		self.bots.iter_mut().enumerate().for_each(|(index, bot)| {
			let context = Context {
				coins: bot.coins,
				cards: bot.cards.clone(),
				..context.clone()
			};
			if !self.playing_bots.contains(&index) {
			} else if bot.name == name {
				let lost_card = bot.interface.on_card_loss(&context);
				if !bot.cards.contains(&lost_card) {
					Self::log(format_args!("🚨  {} is being penalized because \x1b[33mit discarded a card({:?}) it didn't have\x1b[39m", bot, lost_card), self.log);

					let card = bot.cards.pop().unwrap();
					let mut lost_cards = format!("{:?}", card);
					self.discard_pile.push(card);

					if !bot.cards.is_empty() {
						let card = bot.cards.pop().unwrap();
						lost_cards =
							format!("{} and {:?}", lost_cards, card);
						self.discard_pile.push(card);
					}

					bot.cards = vec![];
					Self::log(format_args!("☠️   {} has lost the \x1b[33m{:?}\x1b[39m", bot, lost_cards), self.log);
				} else {
					if let Some(index) = bot.cards.iter().position(|&c| c == lost_card) {
						bot.cards.remove(index);
					}
					self.discard_pile.push(lost_card);

					Self::log(format_args!(
						"{}  {} has lost the \x1b[33m{:?}\x1b[39m",
						if bot.cards.is_empty() {
							"☠️ "
						} else {
							"💔"
						},
						bot,
						lost_card
					), self.log);
				}
			}
		});
	}

	fn penalize_bot(&mut self, name: String, reason: &str) {
		Self::log(
			format_args!(
				"🚨  {} is being penalized because \x1b[33m{}\x1b[39m",
				self.get_bot_by_name(name.clone()),
				reason
			),
			self.log,
		);
		self.card_loss(name);
	}

	fn target_not_found(&self, target: String) -> bool {
		self.bots.iter().filter(|bot| bot.name == target).count() != 1
	}

	fn set_score(&mut self, winners: Vec<String>) {
		let winner_count = winners.len() as f64;
		let loser_count = if self.bots.len() > 6 {
			6.0
		} else {
			self.bots.len() as f64
		} - winner_count;
		let loser_score = -1.0 / loser_count;
		let winner_score = -((loser_score * loser_count) / winner_count);

		self.score = self
			.score
			.iter()
			.map(|(name, score)| {
				if winners.contains(name) {
					(name.clone(), score + winner_score)
				} else {
					(name.clone(), score + loser_score)
				}
			})
			.collect::<Score>();
	}

	// We take a card from a bot and replace it with a new one from the deck
	fn swap_card(&mut self, card: Card, swopee: String) {
		Self::log(
			format_args!(
				"🔄  {} is swapping its card for a new card from the deck",
				self.get_bot_by_name(swopee.clone())
			),
			self.log,
		);
		for bot in self.bots.iter_mut() {
			if bot.name == swopee.clone() {
				if let Some(index) = bot.cards.iter().position(|&c| c == card) {
					bot.cards.remove(index);
				}
				self.deck.push(card);
				self.deck.shuffle(&mut thread_rng());

				let mut new_cards = bot.cards.clone();
				new_cards.push(self.deck.pop().unwrap());
				bot.cards = new_cards;
			}
		}
	}

	/// Playing a game which means we setup the table, give each bots their cards
	/// and coins and start the game loop
	pub fn play(&mut self) {
		self.setup();

		// Logo
		let output = render(Options {
			text: String::from("Coup"),
			colors: vec![Colors::White, Colors::Yellow],
			spaceless: true,
			..Options::default()
		});
		Self::log(
			format_args!(
				"\n\n{}\x1b[4Dv{}\n\n",
				output.text,
				env!("CARGO_PKG_VERSION")
			),
			self.log,
		);

		let bots = self
			.playing_bots
			.iter()
			.map(|bot_index| format!("{}", self.bots[*bot_index]))
			.collect::<Vec<String>>();
		Self::log(
			format_args!("🤺  This rounds player:\n     {}\n", bots.join("\n     "),),
			self.log,
		);

		// Let's play
		while self.playing_bots.len() > 1 {
			self.game_loop();

			if self.moves >= 1000 {
				break;
			}
		}

		let winners = self
			.playing_bots
			.iter()
			.map(|bot_index| self.bots[*bot_index].name.clone())
			.collect::<Vec<String>>();

		self.set_score(winners.clone());

		Self::log(
			format_args!(
				"\n 🎉🎉🎉 The winner{} \x1b[1m{}\x1b[0m \x1b[90min {} moves\x1b[39m\n",
				if winners.len() > 1 { "s are" } else { " is" },
				winners.join(" and "),
				self.moves
			),
			self.log,
		);
	}

	fn game_loop(&mut self) {
		self.moves += 1;

		let context =
			self.get_context(self.bots[self.playing_bots[self.turn]].name.clone());

		// If you have 10 or more coins you must coup
		let action = if self.bots[self.playing_bots[self.turn]].coins >= 10 {
			let target = self.bots[self.playing_bots[self.turn]]
				.interface
				.on_auto_coup(&context);
			Action::Coup(target)
		} else {
			self.bots[self.playing_bots[self.turn]].interface.on_turn(&context)
		};

		match action {
			Action::Assassination(target_name) => {
				if self.target_not_found(target_name.clone()) {
					self.penalize_bot(
						context.name.clone(),
						"it tried to assassinate an unknown bot",
					);
				} else {
					self.history.push(History::ActionAssassination {
						by: context.name.clone(),
						target: target_name.clone(),
					});
					Self::log(
						format_args!(
							"🃏  {} assassinates {} with the \x1b[33mAssassin\x1b[39m",
							self.bots[self.playing_bots[self.turn]],
							self.get_bot_by_name(target_name.clone())
						),
						self.log,
					);
					self.challenge_and_counter_round(
						Action::Assassination(target_name.clone()),
						target_name,
					);
				}
			},
			Action::Coup(target_name) => {
				if self.target_not_found(target_name.clone()) {
					self.penalize_bot(
						context.name.clone(),
						"it tried to coup an unknown bot",
					);
				} else {
					self.history.push(History::ActionCoup {
						by: context.name.clone(),
						target: target_name.clone(),
					});
					Self::log(
						format_args!(
							"🃏  {} \x1b[33mcoups\x1b[39m {}",
							self.bots[self.playing_bots[self.turn]],
							self.get_bot_by_name(target_name.clone())
						),
						self.log,
					);
					self.action_couping(target_name.clone());
				}
			},
			Action::ForeignAid => {
				self.history.push(History::ActionForeignAid {
					by: context.name.clone(),
				});
				Self::log(
					format_args!(
						"🃏  {} takes \x1b[33mforeign aid\x1b[39m",
						self.bots[self.playing_bots[self.turn]],
					),
					self.log,
				);
				self.counter_round_only();
			},
			Action::Swapping => {
				self.history.push(History::ActionSwapping {
					by: context.name.clone(),
				});
				Self::log(
					format_args!(
						"🃏  {} swaps cards with \x1b[33mthe Ambassador\x1b[39m",
						self.bots[self.playing_bots[self.turn]]
					),
					self.log,
				);
				self.challenge_round_only(Action::Swapping);
			},
			Action::Income => {
				self.history.push(History::ActionIncome {
					by: context.name.clone(),
				});
				Self::log(
					format_args!(
						"🃏  {} takes \x1b[33ma coin\x1b[39m",
						self.bots[self.playing_bots[self.turn]]
					),
					self.log,
				);
				self.action_income();
			},
			Action::Stealing(target_name) => {
				if self.target_not_found(target_name.clone()) {
					self.penalize_bot(
						context.name.clone(),
						"it tried to steal from an unknown bot",
					);
				} else {
					self.history.push(History::ActionStealing {
						by: context.name.clone(),
						target: target_name.clone(),
					});
					Self::log(
						format_args!(
							"🃏  {} \x1b[33msteals 2 coins\x1b[39m from {}",
							self.bots[self.playing_bots[self.turn]],
							self.get_bot_by_name(target_name.clone()),
						),
						self.log,
					);
					self.challenge_and_counter_round(
						Action::Stealing(target_name.clone()),
						target_name,
					);
				}
			},
			Action::Tax => {
				self.history.push(History::ActionTax {
					by: context.name.clone(),
				});
				Self::log(
					format_args!(
						"🃏  {} takes tax with the \x1b[33mDuke\x1b[39m",
						self.bots[self.playing_bots[self.turn]],
					),
					self.log,
				);
				self.challenge_round_only(Action::Tax);
			},
		}

		// Let's filter out all dead bots
		self.playing_bots = self
			.playing_bots
			.iter()
			.filter(|bot_index| !self.bots[**bot_index].cards.is_empty())
			.copied()
			.collect::<Vec<usize>>();

		// We move to the next turn
		self.turn = if self.playing_bots.is_empty()
			|| self.turn >= self.playing_bots.len() - 1
		{
			0
		} else {
			self.turn + 1
		};
	}

	fn challenge_and_counter_round(
		&mut self,
		action: Action,
		target_name: String,
	) {
		// THE CHALLENGE ROUND
		let playing_bot_name = self.bots[self.playing_bots[self.turn]].name.clone();
		// On Action::Assassination and Action::Stealing
		// Does anyone want to challenge this action?
		if let Some(challenger) = self.challenge_round(
			ChallengeRound::Action,
			&action,
			playing_bot_name.clone(),
		) {
			// The bot "challenger" is challenging this action
			let success = self.resolve_challenge(
				action.clone(),
				playing_bot_name.clone(),
				challenger.clone(),
			);
			if !success {
				// The challenge was unsuccessful
				// Discard the card and pick up a new card from the deck
				let discard_card = match action {
					Action::Assassination(_) => Card::Assassin,
					Action::Stealing(_) => Card::Captain,
					Action::Coup(_)
					| Action::ForeignAid
					| Action::Swapping
					| Action::Income
					| Action::Tax => {
						unreachable!("Challenge and counter not called on other actions")
					},
				};
				self.swap_card(discard_card, playing_bot_name.clone());
			} else {
				// The challenge was successful so we stop a counter round
				return;
			}
		}

		// At this point it's possible this bot is dead already and can't
		// play any counters.
		// Scenario:
		// - Bot1(1 card) gets assassinated by Bot2
		// - Bot1(1 card) challenges this assassination unsuccessfully
		// - Bot1(0 card) is now dead and can't counter
		if !self.get_bot_by_name(target_name.clone()).cards.is_empty() {
			// THE COUNTER CHALLENGE ROUND
			// Does the target want to counter this action?
			let counter =
				self.get_bot_by_name(target_name.clone()).interface.on_counter(
					&action,
					playing_bot_name.clone(),
					&self.get_context(target_name.clone()),
				);

			if counter.is_some() {
				// The bot target_name is countering the action so we now ask the
				// table if anyone would like to challenge this counter
				match action {
					Action::Assassination(_) => {
						self.history.push(History::CounterAssassination {
							by: target_name.clone(),
							target: playing_bot_name.clone(),
						})
					},
					Action::Stealing(_) => self.history.push(History::CounterStealing {
						by: target_name.clone(),
						target: playing_bot_name.clone(),
					}),
					Action::Coup(_)
					| Action::ForeignAid
					| Action::Swapping
					| Action::Income
					| Action::Tax => {
						unreachable!("Challenge and counter not called on other actions")
					},
				};
				Self::log(
					format_args!(
						"🛑  {} was countered by {}",
						self.get_bot_by_name(playing_bot_name.clone()),
						self.get_bot_by_name(target_name.clone()),
					),
					self.log,
				);

				if let Some(counter_challenge) = self.challenge_round(
					ChallengeRound::Counter,
					&action,
					target_name.clone(),
				) {
					let counter_card = match action {
						Action::Assassination(_) => Counter::Assassination,
						Action::Stealing(_) => Counter::Stealing,
						Action::Coup(_)
						| Action::ForeignAid
						| Action::Swapping
						| Action::Income
						| Action::Tax => {
							unreachable!("Challenge and counter not called on other actions")
						},
					};
					// The bot counter_challenge.by is challenging this action
					let success = self.resolve_counter_challenge(
						counter_card,
						target_name.clone(),
						counter_challenge.clone(),
					);
					if success {
						// The challenge was successful so the player who countered gets a
						// penalty but the action is still performed
						match action {
							Action::Assassination(_) => {
								self.action_assassination(target_name.clone())
							},
							Action::Stealing(_) => self.action_stealing(target_name.clone()),
							Action::Coup(_)
							| Action::ForeignAid
							| Action::Swapping
							| Action::Income
							| Action::Tax => unreachable!(
								"Challenge and counter not called on other actions"
							),
						}
					}
				} else {
					// There was no challenge to the counter played so the action is
					// not performed (because it is countered).
				}
			} else {
				// No counter was played so the action is performed
				match action {
					Action::Assassination(_) => {
						self.action_assassination(target_name.clone())
					},
					Action::Stealing(_) => self.action_stealing(target_name.clone()),
					Action::Coup(_)
					| Action::ForeignAid
					| Action::Swapping
					| Action::Income
					| Action::Tax => {
						unreachable!("Challenge and counter not called on other actions")
					},
				}
			}
		}
	}

	fn challenge_round_only(&mut self, action: Action) {
		// THE CHALLENGE ROUND
		let playing_bot_name = self.bots[self.playing_bots[self.turn]].name.clone();
		// On Action::Swapping and Action::Tax
		// Does anyone want to challenge this action?
		if let Some(challenger) = self.challenge_round(
			ChallengeRound::Action,
			&action,
			playing_bot_name.clone(),
		) {
			// The bot "challenger" is challenging this action
			let success = self.resolve_challenge(
				action.clone(),
				playing_bot_name.clone(),
				challenger.clone(),
			);
			if !success {
				// The challenge was unsuccessful
				// Discard the card and pick up a new card from the deck
				let discard_card = match action {
					Action::Swapping => Card::Ambassador,
					Action::Tax => Card::Duke,
					Action::Coup(_)
					| Action::Assassination(_)
					| Action::ForeignAid
					| Action::Income
					| Action::Stealing(_) => {
						unreachable!("Challenge only not called on other actions")
					},
				};
				self.swap_card(discard_card, playing_bot_name.clone());

				// The challenge was unsuccessful so let's do the thing
				match action {
					Action::Swapping => self.action_swapping(),
					Action::Tax => self.action_tax(),
					Action::Coup(_)
					| Action::Assassination(_)
					| Action::ForeignAid
					| Action::Income
					| Action::Stealing(_) => {
						unreachable!("Challenge only not called on other actions")
					},
				}
			}
		} else {
			// No challenge was played so the action is performed
			match action {
				Action::Swapping => self.action_swapping(),
				Action::Tax => self.action_tax(),
				Action::Coup(_)
				| Action::Assassination(_)
				| Action::ForeignAid
				| Action::Income
				| Action::Stealing(_) => {
					unreachable!("Challenge only not called on other actions")
				},
			}
		}
	}

	fn counter_round_only(&mut self) {
		let playing_bot_name = self.bots[self.playing_bots[self.turn]].name.clone();
		// THE COUNTER CHALLENGE ROUND
		// On Action::ForeignAid
		// Does anyone want to counter this action?
		let mut counterer_name = String::new();
		for bot_index in self.playing_bots.iter() {
			let bot = &self.bots[*bot_index];
			// Skipping the challenger
			if bot.name.clone() == playing_bot_name.clone() {
				continue;
			}

			let countering = bot.interface.on_counter(
				&Action::ForeignAid,
				playing_bot_name.clone(),
				&self.get_context(playing_bot_name.clone()),
			);

			if countering.is_some() {
				counterer_name = bot.name.clone();
				break;
			}
		}

		if !counterer_name.is_empty() {
			self.history.push(History::CounterForeignAid {
				by: counterer_name.clone(),
				target: playing_bot_name.clone(),
			});
			Self::log(
				format_args!(
					"🛑  {} was countered by {}",
					self.get_bot_by_name(playing_bot_name.clone()),
					self.get_bot_by_name(counterer_name.clone()),
				),
				self.log,
			);

			// The bot counterer_name is countering the action so we now ask the table
			// if anyone would like to challenge this counter
			if let Some(counter_challenge) = self.challenge_round(
				ChallengeRound::Counter,
				&Action::ForeignAid,
				counterer_name.clone(),
			) {
				// The bot counter_challenge.by is challenging this action
				let success = self.resolve_counter_challenge(
					Counter::ForeignAid,
					counterer_name.clone(),
					counter_challenge.clone(),
				);
				if success {
					self.action_foraign_aid();
				}
			}
		} else {
			// No counter was played so the action is performed
			self.action_foraign_aid();
		}
	}

	// All bots (minus the playing bot) are asked if they want to challenge a play
	fn challenge_round(
		&mut self,
		challenge_type: ChallengeRound,
		action: &Action,
		by: String,
	) -> Option<String> {
		for bot_index in self.playing_bots.iter() {
			let bot = &self.bots[*bot_index];
			// skipping the challenger
			if bot.name.clone() == by.clone() {
				continue;
			}

			let context = self.get_context(bot.name.clone());

			let challenging = match challenge_type {
				ChallengeRound::Action => {
					bot.interface.on_challenge_action_round(action, by.clone(), &context)
				},
				ChallengeRound::Counter => {
					bot.interface.on_challenge_counter_round(action, by.clone(), &context)
				},
			};

			if challenging {
				Self::log(
					format_args!(
						"❓  {} was challenged by {}",
						self.get_bot_by_name(by),
						bot
					),
					self.log,
				);
				return Some(bot.name.clone());
			}
		}
		None
	}

	// Someone challenged another bot for playing a card they believe is a bluff
	fn resolve_challenge(
		&mut self,
		action: Action,
		player: String,
		challenger: String,
	) -> bool {
		self.history.push(match action {
			Action::Assassination(_) => History::ChallengeAssassin {
				by: challenger.clone(),
				target: player.clone(),
			},
			Action::Swapping => History::ChallengeAmbassador {
				by: challenger.clone(),
				target: player.clone(),
			},
			Action::Stealing(_) => History::ChallengeCaptain {
				by: challenger.clone(),
				target: player.clone(),
			},
			Action::Tax => History::ChallengeDuke {
				by: challenger.clone(),
				target: player.clone(),
			},
			Action::Coup(_) | Action::Income | Action::ForeignAid => {
				unreachable!("Can't challenge Coup, Income or ForeignAid")
			},
		});

		let player = self.get_bot_by_name(player.clone());
		let challenger = self.get_bot_by_name(challenger.clone());

		let card = match action {
			Action::Assassination(_) => Card::Assassin,
			Action::Swapping => Card::Ambassador,
			Action::Stealing(_) => Card::Captain,
			Action::Tax => Card::Duke,
			Action::Coup(_) | Action::Income | Action::ForeignAid => {
				unreachable!("Can't challenge Coup, Income or ForeignAid")
			},
		};

		if player.cards.contains(&card) {
			Self::log(
				format_args!(
					"👎  The challenge was unsuccessful because {} \x1b[33mdid have the {:?}\x1b[39m",
					player, card
				),
				self.log,
			);
			self.card_loss(challenger.name.clone());
			false
		} else {
			Self::log(
				format_args!(
					"👍  The challenge was successful because {} \x1b[33mdidn't have the {:?}\x1b[39m",
					player, card
				),
				self.log,
			);
			self.card_loss(player.name.clone());
			true
		}
	}

	// A bot is countering another bots action against them
	fn resolve_counter_challenge(
		&mut self,
		counter: Counter,
		counterer: String,
		challenger: String,
	) -> bool {
		self.history.push(match counter {
			Counter::Assassination => History::CounterChallengeContessa {
				by: challenger.clone(),
				target: counterer.clone(),
			},
			Counter::ForeignAid => History::CounterChallengeDuke {
				by: challenger.clone(),
				target: counterer.clone(),
			},
			Counter::Stealing => History::CounterChallengeCaptainAmbassedor {
				by: challenger.clone(),
				target: counterer.clone(),
			},
		});

		let counterer = self.get_bot_by_name(counterer.clone());
		let challenger = self.get_bot_by_name(challenger.clone());

		let cards = match counter {
			Counter::Assassination => vec![Card::Contessa],
			Counter::ForeignAid => vec![Card::Duke],
			Counter::Stealing => vec![Card::Captain, Card::Ambassador],
		};
		let card_string = cards
			.iter()
			.map(|card| format!("{:?}", card))
			.collect::<Vec<String>>()
			.join(" or the ");

		if cards.iter().any(|&card| counterer.cards.contains(&card)) {
			Self::log(
				format_args!(
					"👎  The counter was unsuccessful because {} \x1b[33mdid have the {}\x1b[39m",
					counterer, card_string
				),
				self.log,
			);
			self.card_loss(challenger.name.clone());
			false
		} else {
			Self::log(
				format_args!(
					"👍  The counter was successful because {} \x1b[33mdidn't have the {}\x1b[39m",
					counterer, card_string
				),
				self.log,
			);
			self.card_loss(counterer.name.clone());
			true
		}
	}

	fn display_score(&mut self) {
		let fps = (self.rounds as f64 / 1000.0).max(1.0) as u64;
		if self.round == 0 || self.round % fps == 0 || self.round + 1 == self.rounds
		{
			if self.round > 0 {
				print!("\x1b[{}A\x1b[2K", self.score.len() + 1);
			}

			let done =
				(((self.round + 1) as f64 / self.rounds as f64) * 100.0).round();
			println!("\x1b[2K {:>3}% done", done);
			self.score.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
			self.score.iter().for_each(|(name, score)| {
				let percentage = if self.round > 0 {
					format!("{:.3}", (score * 100.0) / self.round as f64)
				} else { String::from("0") };
				println!("\x1b[2K\x1b[90m {:>8}%\x1b[39m  \x1b[31m{:>15.5}\x1b[39m  \x1b[33m{}\x1b[39m", percentage, score, name);
			});
		}
	}

	fn format_number_with_separator(mut number: u64) -> String {
		if number == 0 {
			return String::from("0");
		}
		let mut result = String::new();
		let mut count = 0;

		while number != 0 {
			if count % 3 == 0 && count != 0 {
				result.insert(0, ',');
			}
			count += 1;
			result.insert(0, (b'0' + (number % 10) as u8) as char);
			number /= 10;
		}

		result
	}

	/// Play n number of rounds and tally up the score in the CLI
	pub fn looping(&mut self, rounds: u64) {
		self.setup();
		self.log = false;
		self.rounds = rounds;

		// Logo
		let output = render(Options {
			text: String::from("Coup"),
			colors: vec![Colors::White, Colors::Yellow],
			spaceless: true,
			..Options::default()
		});
		println!("\n\n{}\x1b[4Dv{}\n\n", output.text, env!("CARGO_PKG_VERSION"));

		println!(
			" Starting \x1b[36m{}\x1b[39m rounds",
			Self::format_number_with_separator(rounds)
		);

		println!(" ╔═════════════════╗\n ║ 🎲🎲 \x1b[1mBOARD\x1b[0m 🎲🎲 ║\n ╚═════════════════╝\x1b[?25l");
		self.display_score();
		for round in 0..rounds {
			self.setup();
			self.play();
			// TODO: detect "stop" and record log in debug mode
			self.round = round + 1;
			self.display_score();
		}

		println!(
			"\x1b[?25h\n 🎉🎉🎉 The winner is: \x1b[1m{}\x1b[0m\n",
			self
				.score
				.iter()
				.max_by(|(_, a), (_, b)| a
					.partial_cmp(b)
					.unwrap_or(std::cmp::Ordering::Equal))
				.unwrap()
				.0
		);
	}

	// *******************************| Actions |****************************** //
	fn action_assassination(&mut self, target: String) {
		let playing_bot_coins = self.bots[self.playing_bots[self.turn]].coins;
		let playing_bot_name = self.bots[self.playing_bots[self.turn]].name.clone();
		if playing_bot_coins < 3 {
			self.penalize_bot(
				playing_bot_name.clone(),
				"it tried to assassinate someone with insufficient funds",
			);
		} else if self.target_not_found(target.clone()) {
			self.penalize_bot(
				playing_bot_name.clone(),
				"it tried to assassinate an unknown bot",
			);
		} else {
			// Paying the fee
			self.bots[self.playing_bots[self.turn]].coins = playing_bot_coins - 3;

			// Taking a card from the target bot
			self.card_loss(target);
		}
	}

	fn action_couping(&mut self, target: String) {
		let playing_bot_coins = self.bots[self.playing_bots[self.turn]].coins;
		let playing_bot_name = self.bots[self.playing_bots[self.turn]].name.clone();
		if playing_bot_coins < 7 {
			self.penalize_bot(
				playing_bot_name.clone(),
				"it tried to coup someone with insufficient funds",
			);
		} else if self.target_not_found(target.clone()) {
			self.penalize_bot(
				playing_bot_name.clone(),
				"it tried to coup an unknown bot",
			);
		} else {
			// Paying the fee
			self.bots[self.playing_bots[self.turn]].coins = playing_bot_coins - 7;

			// Taking a card from the target bot
			self.card_loss(target);
		}
	}

	fn action_foraign_aid(&mut self) {
		let coins = self.bots[self.playing_bots[self.turn]].coins;
		self.bots[self.playing_bots[self.turn]].coins = coins + 2;
	}

	fn action_swapping(&mut self) {
		let mut all_available_cards =
			self.bots[self.playing_bots[self.turn]].cards.clone();
		let card1 = self.deck.pop().unwrap();
		let card2 = self.deck.pop().unwrap();
		let cards_from_deck = [card1, card2];
		let swapped_cards =
			self.bots[self.playing_bots[self.turn]].interface.on_swapping_cards(
				cards_from_deck,
				&self.get_context(self.bots[self.playing_bots[self.turn]].name.clone()),
			);
		all_available_cards.push(card1);
		all_available_cards.push(card2);

		if !(all_available_cards.contains(&swapped_cards[0])
			&& all_available_cards.contains(&swapped_cards[1]))
		{
			self.penalize_bot(
				self.bots[self.playing_bots[self.turn]].name.clone(),
				"it tried to swap cards it didn't have",
			);
		} else {
			self.deck.push(swapped_cards[0]);
			self.deck.push(swapped_cards[1]);
			self.deck.shuffle(&mut thread_rng());

			// removing the discarded cards from the pool and giving it to the bot
			if let Some(index) =
				all_available_cards.iter().position(|&c| c == swapped_cards[0])
			{
				all_available_cards.remove(index);
			}
			if let Some(index) =
				all_available_cards.iter().position(|&c| c == swapped_cards[1])
			{
				all_available_cards.remove(index);
			}
			self.bots[self.playing_bots[self.turn]].cards = all_available_cards;
		}
	}

	fn action_income(&mut self) {
		let playing_bot_coins = self.bots[self.playing_bots[self.turn]].coins;
		self.bots[self.playing_bots[self.turn]].coins = playing_bot_coins + 1;
	}

	fn action_stealing(&mut self, target: String) {
		let coins = self.bots[self.playing_bots[self.turn]].coins;
		let target_coins = self.get_bot_by_name(target.clone()).coins;
		let booty = std::cmp::min(target_coins, 2);
		self.bots[self.playing_bots[self.turn]].coins = coins + booty;
		self
			.bots
			.iter_mut()
			.find(|bot| bot.name.clone() == target)
			.unwrap()
			.coins = target_coins - booty;
	}

	fn action_tax(&mut self) {
		let coins = self.bots[self.playing_bots[self.turn]].coins;
		self.bots[self.playing_bots[self.turn]].coins = coins + 3;
	}
}

/// The debug trait has been implemented to support both format and alternate
/// format which means you can print a game instance with:
/// ```rust
/// # use coup::Coup;
/// let mut my_coup = Coup::new(vec![]);
/// println!("{:?}", my_coup);
/// ```
/// and
/// ```rust
/// # use coup::Coup;
/// let mut my_coup = Coup::new(vec![]);
/// println!("{:#?}", my_coup);
/// ```
impl fmt::Debug for Coup {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			writeln!(f, "Coup {{")?;
			writeln!(f, "  bots: {:#?}", self.bots)?;
			writeln!(f, "  playing_bots: {:#?}", self.playing_bots)?;
			writeln!(f, "  deck: {:#?}", self.deck)?;
			writeln!(f, "  discard_pile: {:#?}", self.discard_pile)?;
			writeln!(f, "  history: {:#?}", self.history)?;
			writeln!(f, "  score: {:#?}", self.score)?;
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::bots::StaticBot;

	#[test]
	fn test_new() {
		let coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);

		assert_eq!(coup.bots[0].cards, vec![]);
		assert_eq!(coup.bots[1].cards, vec![]);
		assert_eq!(coup.playing_bots, Vec::<usize>::new());
		assert_eq!(coup.deck, vec![]);
		assert_eq!(coup.discard_pile, vec![]);
		assert_eq!(coup.history, vec![]);
		assert_eq!(
			coup.score,
			vec![
				(String::from("StaticBot"), 0.0),
				(String::from("StaticBot 2"), 0.0)
			]
		);
		assert_eq!(coup.turn, 0);
		assert_eq!(coup.moves, 0);
	}

	#[test]
	fn test_new_deck() {
		let deck = Coup::new_deck();
		assert_eq!(deck.len(), 15);
		assert_eq!(
			deck.iter().filter(|&card| card == &Card::Ambassador).count(),
			3
		);
		assert_eq!(deck.iter().filter(|&card| card == &Card::Assassin).count(), 3);
		assert_eq!(deck.iter().filter(|&card| card == &Card::Captain).count(), 3);
		assert_eq!(deck.iter().filter(|&card| card == &Card::Contessa).count(), 3);
		assert_eq!(deck.iter().filter(|&card| card == &Card::Duke).count(), 3);
	}

	#[test]
	fn test_setup() {
		let mut coup = Coup::new(vec![
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
		]);
		coup.setup();

		assert_eq!(coup.bots[coup.playing_bots[0]].cards.len(), 2);
		assert_eq!(coup.bots[coup.playing_bots[0]].coins, 2);
		assert_eq!(coup.bots[coup.playing_bots[1]].cards.len(), 2);
		assert_eq!(coup.bots[coup.playing_bots[1]].coins, 2);
		assert_eq!(coup.bots[coup.playing_bots[2]].cards.len(), 2);
		assert_eq!(coup.bots[coup.playing_bots[2]].coins, 2);
		assert_eq!(coup.bots[coup.playing_bots[3]].cards.len(), 2);
		assert_eq!(coup.bots[coup.playing_bots[3]].coins, 2);
		assert_eq!(coup.bots[coup.playing_bots[4]].cards.len(), 2);
		assert_eq!(coup.bots[coup.playing_bots[4]].coins, 2);
		assert_eq!(coup.bots[coup.playing_bots[5]].cards.len(), 2);
		assert_eq!(coup.bots[coup.playing_bots[5]].coins, 2);
		assert_eq!(coup.playing_bots.len(), 6);
		assert_eq!(coup.deck.len(), 3);
		assert_eq!(coup.discard_pile, vec![]);
		assert_eq!(coup.turn, 0);
		assert_eq!(coup.moves, 0);
	}

	// TODO: test_log

	#[test]
	fn test_get_bot_by_name() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		assert_eq!(
			coup.get_bot_by_name(String::from("StaticBot")).name,
			String::from("StaticBot")
		);
		assert_eq!(
			coup.get_bot_by_name(String::from("StaticBot 2")).name,
			String::from("StaticBot 2")
		);
	}

	#[test]
	fn test_get_other_bots() {
		let mut coup = Coup::new(vec![
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
		]);
		coup.setup();

		coup.playing_bots = vec![0, 1, 2, 3, 4];
		coup.turn = 0;
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot 2"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot 3"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot 4"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot 5"),
			coins: 2,
			cards: 2
		}));

		coup.playing_bots = vec![4, 3, 2, 1, 0];
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot 2"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot 3"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot 4"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot 5"),
			coins: 2,
			cards: 2
		}));

		coup.turn = 2;
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot 2"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot 3"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot 4"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot 5"),
			coins: 2,
			cards: 2
		}));

		coup.bots[0].cards = vec![];
		coup.bots[1].cards = vec![Card::Duke];
		coup.bots[3].cards = vec![];
		coup.playing_bots = vec![1, 2, 4];
		coup.turn = 2;
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot 2"),
			coins: 2,
			cards: 1,
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("StaticBot 3"),
			coins: 2,
			cards: 2,
		}));
	}

	#[test]
	fn test_get_context() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		coup.bots[0].cards = vec![Card::Ambassador, Card::Duke];
		coup.bots[1].cards = vec![Card::Captain, Card::Captain];
		coup.playing_bots = vec![0, 1];

		assert_eq!(
			coup.get_context(String::from("StaticBot")),
			Context {
				name: String::from("StaticBot"),
				coins: 2,
				cards: vec![Card::Ambassador, Card::Duke],
				playing_bots: vec![
					OtherBot {
						name: String::from("StaticBot"),
						coins: 2,
						cards: 2
					},
					OtherBot {
						name: String::from("StaticBot 2"),
						coins: 2,
						cards: 2
					}
				],
				discard_pile: vec![],
				history: vec![],
				score: vec![
					(String::from("StaticBot"), 0.0),
					(String::from("StaticBot 2"), 0.0)
				],
			}
		);

		coup.turn = 1;
		assert_eq!(
			coup.get_context(String::from("StaticBot 2")),
			Context {
				name: String::from("StaticBot 2"),
				coins: 2,
				cards: vec![Card::Captain, Card::Captain],
				playing_bots: vec![
					OtherBot {
						name: String::from("StaticBot"),
						coins: 2,
						cards: 2
					},
					OtherBot {
						name: String::from("StaticBot 2"),
						coins: 2,
						cards: 2
					}
				],
				discard_pile: vec![],
				history: vec![],
				score: vec![
					(String::from("StaticBot"), 0.0),
					(String::from("StaticBot 2"), 0.0)
				],
			}
		);
	}

	#[test]
	fn test_card_loss() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		coup.bots[0].cards = vec![Card::Ambassador, Card::Duke];
		coup.bots[1].cards = vec![Card::Captain, Card::Captain];

		coup.card_loss(String::from("StaticBot 2"));

		assert_eq!(coup.bots[0].cards, vec![Card::Ambassador, Card::Duke]);
		assert_eq!(coup.bots[1].cards, vec![Card::Captain]);
		assert_eq!(coup.discard_pile, vec![Card::Captain]);
	}

	#[test]
	fn test_card_loss_faulty_bot() {
		struct TestBot;
		impl BotInterface for TestBot {
			fn get_name(&self) -> String {
				String::from("TestBot")
			}
			fn on_card_loss(&self, _context: &Context) -> Card {
				Card::Duke
			}
		}

		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(TestBot)]);
		coup.setup();

		coup.bots[0].cards = vec![Card::Ambassador, Card::Duke];
		coup.bots[1].cards = vec![Card::Assassin, Card::Captain];

		coup.card_loss(String::from("TestBot"));

		assert_eq!(coup.bots[0].cards, vec![Card::Ambassador, Card::Duke]);
		assert_eq!(coup.bots[1].cards, vec![]);
		assert_eq!(coup.discard_pile, vec![Card::Captain, Card::Assassin]);
	}

	// TODO: test_penalize_bot

	#[test]
	fn test_target_not_found() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		assert_eq!(coup.target_not_found(String::from("StaticBot")), false);
		assert_eq!(coup.target_not_found(String::from("StaticBot 3")), true);
		assert_eq!(coup.target_not_found(String::from("StaticBot 2")), false);
	}

	#[test]
	fn test_set_score() {
		// Two players, one winner
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		coup.set_score(vec![String::from("StaticBot")]);

		assert_eq!(
			coup.score,
			vec![
				(String::from("StaticBot"), 1.0),
				(String::from("StaticBot 2"), -1.0)
			]
		);

		// Five players, one winner
		coup = Coup::new(vec![
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
		]);
		coup.setup();

		coup.set_score(vec![String::from("StaticBot")]);

		assert_eq!(
			coup.score,
			vec![
				(String::from("StaticBot"), 1.0),
				(String::from("StaticBot 2"), -0.25),
				(String::from("StaticBot 3"), -0.25),
				(String::from("StaticBot 4"), -0.25),
				(String::from("StaticBot 5"), -0.25),
			]
		);

		// Five players, two winner
		coup = Coup::new(vec![
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
		]);
		coup.setup();

		coup
			.set_score(vec![String::from("StaticBot"), String::from("StaticBot 2")]);

		assert_eq!(
			coup.score,
			vec![
				(String::from("StaticBot"), 0.5),
				(String::from("StaticBot 2"), 0.5),
				(String::from("StaticBot 3"), -0.3333333333333333),
				(String::from("StaticBot 4"), -0.3333333333333333),
				(String::from("StaticBot 5"), -0.3333333333333333),
			]
		);
	}

	#[test]
	fn test_swap_card() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();
		coup.playing_bots = vec![0, 1];

		coup.bots[0].cards = vec![Card::Ambassador, Card::Ambassador];
		coup.bots[1].cards = vec![Card::Assassin, Card::Captain];
		coup.deck = vec![Card::Ambassador, Card::Captain];

		assert_eq!(coup.bots[0].cards, vec![Card::Ambassador, Card::Ambassador]);
		assert_eq!(coup.bots[1].cards, vec![Card::Assassin, Card::Captain]);
		assert_eq!(coup.deck, vec![Card::Ambassador, Card::Captain]);
		assert_eq!(coup.discard_pile, vec![]);

		coup.swap_card(Card::Ambassador, String::from("StaticBot"));

		assert_eq!(coup.bots[0].cards.len(), 2);
		assert_eq!(coup.bots[1].cards, vec![Card::Assassin, Card::Captain]);
		assert_eq!(coup.deck.len(), 2);
		assert_eq!(coup.discard_pile, vec![]);
	}

	// TODO: test_play
	// TODO: test_game_loop

	#[test]
	fn test_challenge_and_counter_round_assassination() {
		struct ActionChallengeBot;
		impl BotInterface for ActionChallengeBot {
			fn get_name(&self) -> String {
				String::from("ActionChallengeBot")
			}
			fn on_challenge_action_round(
				&self,
				_action: &Action,
				_by: String,
				_context: &Context,
			) -> bool {
				true
			}
		}
		struct ChallengeCounterBot;
		impl BotInterface for ChallengeCounterBot {
			fn get_name(&self) -> String {
				String::from("ChallengeCounterBot")
			}
			fn on_challenge_counter_round(
				&self,
				_action: &Action,
				_by: String,
				_context: &Context,
			) -> bool {
				true
			}
		}
		struct CounterBot;
		impl BotInterface for CounterBot {
			fn get_name(&self) -> String {
				String::from("CounterBot")
			}
			fn on_counter(
				&self,
				_action: &Action,
				_by: String,
				_context: &Context,
			) -> Option<bool> {
				Some(true)
			}
		}

		// Successful challenge
		let mut coup = Coup::new(vec![
			Box::new(StaticBot),
			Box::new(ActionChallengeBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
		]);
		coup.setup();
		coup.bots[0].cards = vec![Card::Duke, Card::Captain];
		coup.bots[0].coins = 4;
		coup.bots[3].cards = vec![Card::Ambassador, Card::Assassin];
		coup.playing_bots = vec![0, 1, 2, 3, 4, 5];
		coup.turn = 0;
		coup.history = vec![];

		coup.challenge_and_counter_round(
			Action::Assassination(String::from("StaticBot 2")),
			String::from("StaticBot 2"),
		);

		assert_eq!(coup.bots[0].cards, vec![Card::Duke]);
		assert_eq!(coup.bots[0].coins, 4);
		assert_eq!(coup.bots[1].cards.len(), 2);
		assert_eq!(coup.bots[2].cards.len(), 2);
		assert_eq!(coup.bots[3].cards.len(), 2);
		assert_eq!(coup.bots[4].cards.len(), 2);
		assert_eq!(coup.bots[5].cards.len(), 2);
		assert_eq!(
			coup.history,
			vec![History::ChallengeAssassin {
				by: String::from("ActionChallengeBot"),
				target: String::from("StaticBot"),
			}]
		);

		// Successful counter
		coup = Coup::new(vec![
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(CounterBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
		]);
		coup.setup();
		coup.bots[0].cards = vec![Card::Duke, Card::Captain];
		coup.bots[0].coins = 4;
		coup.bots[3].cards = vec![Card::Ambassador, Card::Assassin];
		coup.playing_bots = vec![0, 1, 2, 3, 4, 5];
		coup.turn = 0;
		coup.history = vec![];

		coup.challenge_and_counter_round(
			Action::Assassination(String::from("CounterBot")),
			String::from("CounterBot"),
		);

		assert_eq!(coup.bots[0].cards, vec![Card::Duke, Card::Captain]);
		assert_eq!(coup.bots[0].coins, 4);
		assert_eq!(coup.bots[1].cards.len(), 2);
		assert_eq!(coup.bots[2].cards.len(), 2);
		assert_eq!(coup.bots[3].cards, vec![Card::Ambassador, Card::Assassin]);
		assert_eq!(coup.bots[4].cards.len(), 2);
		assert_eq!(coup.bots[5].cards.len(), 2);
		assert_eq!(
			coup.history,
			vec![History::CounterAssassination {
				by: String::from("CounterBot"),
				target: String::from("StaticBot"),
			}]
		);

		// Successful counter challenge
		coup = Coup::new(vec![
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(CounterBot),
			Box::new(StaticBot),
			Box::new(ChallengeCounterBot),
		]);
		coup.setup();
		coup.bots[0].cards = vec![Card::Duke, Card::Captain];
		coup.bots[0].coins = 4;
		coup.bots[3].cards = vec![Card::Captain, Card::Assassin];
		coup.playing_bots = vec![0, 1, 2, 3, 4, 5];
		coup.turn = 0;
		coup.history = vec![];

		coup.challenge_and_counter_round(
			Action::Assassination(String::from("CounterBot")),
			String::from("CounterBot"),
		);

		assert_eq!(coup.bots[0].cards, vec![Card::Duke, Card::Captain]);
		assert_eq!(coup.bots[0].coins, 1);
		assert_eq!(coup.bots[1].cards.len(), 2);
		assert_eq!(coup.bots[2].cards.len(), 2);
		assert_eq!(coup.bots[3].cards.len(), 0);
		assert_eq!(coup.bots[4].cards.len(), 2);
		assert_eq!(coup.bots[5].cards.len(), 2);
		assert_eq!(
			coup.history,
			vec![
				History::CounterAssassination {
					by: String::from("CounterBot"),
					target: String::from("StaticBot"),
				},
				History::CounterChallengeContessa {
					by: String::from("ChallengeCounterBot"),
					target: String::from("CounterBot"),
				}
			]
		);

		// Successful action
		coup = Coup::new(vec![
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
		]);
		coup.setup();
		coup.bots[0].cards = vec![Card::Duke, Card::Captain];
		coup.bots[0].coins = 4;
		coup.bots[3].cards = vec![Card::Ambassador, Card::Assassin];
		coup.playing_bots = vec![0, 1, 2, 3, 4, 5];
		coup.turn = 0;
		coup.history = vec![];

		coup.challenge_and_counter_round(
			Action::Assassination(String::from("StaticBot 4")),
			String::from("StaticBot 4"),
		);

		assert_eq!(coup.bots[0].cards, vec![Card::Duke, Card::Captain]);
		assert_eq!(coup.bots[0].coins, 1);
		assert_eq!(coup.bots[1].cards.len(), 2);
		assert_eq!(coup.bots[2].cards.len(), 2);
		assert_eq!(coup.bots[3].cards, vec![Card::Ambassador]);
		assert_eq!(coup.bots[4].cards.len(), 2);
		assert_eq!(coup.bots[5].cards.len(), 2);

		// Unsuccessful challenge
		// Unsuccessful counter
		// Unsuccessful counter challenge
	}

	#[test]
	fn test_challenge_round_only_successful() {
		// Swapping
		struct TestBot;
		impl BotInterface for TestBot {
			fn get_name(&self) -> String {
				String::from("TestBot")
			}
			fn on_challenge_action_round(
				&self,
				_action: &Action,
				_by: String,
				_context: &Context,
			) -> bool {
				true
			}
		}

		let mut coup = Coup::new(vec![
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(TestBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
		]);
		coup.setup();
		coup.bots[0].cards = vec![Card::Duke, Card::Captain];
		coup.bots[2].cards = vec![Card::Ambassador, Card::Assassin];
		coup.playing_bots = vec![0, 1, 2, 3, 4];
		coup.turn = 0;

		coup.challenge_round_only(Action::Swapping);

		assert_eq!(coup.bots[0].cards, vec![Card::Duke]);
		assert_eq!(coup.bots[2].cards, vec![Card::Ambassador, Card::Assassin]);

		// Action::Tax
		coup.setup();
		coup.bots[0].cards = vec![Card::Ambassador, Card::Captain];
		coup.bots[2].cards = vec![Card::Ambassador, Card::Assassin];
		coup.playing_bots = vec![0, 1, 2, 3, 4];
		coup.turn = 0;

		coup.challenge_round_only(Action::Tax);

		assert_eq!(coup.bots[0].cards, vec![Card::Ambassador]);
		assert_eq!(coup.bots[2].cards, vec![Card::Ambassador, Card::Assassin]);
	}

	#[test]
	fn test_challenge_round_only_unsuccessful() {
		// Swapping
		struct TestBot;
		impl BotInterface for TestBot {
			fn get_name(&self) -> String {
				String::from("TestBot")
			}
			fn on_challenge_action_round(
				&self,
				_action: &Action,
				_by: String,
				_context: &Context,
			) -> bool {
				true
			}
		}

		let mut coup = Coup::new(vec![
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(TestBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
		]);
		coup.setup();
		coup.bots[0].cards = vec![Card::Ambassador, Card::Captain];
		coup.bots[2].cards = vec![Card::Ambassador, Card::Assassin];
		coup.playing_bots = vec![0, 1, 2, 3, 4];
		coup.turn = 0;

		coup.challenge_round_only(Action::Swapping);

		assert_eq!(coup.bots[0].cards.len(), 2);
		assert_eq!(coup.bots[2].cards, vec![Card::Ambassador]);

		// Action::Tax
		coup.setup();
		coup.bots[0].cards = vec![Card::Duke, Card::Captain];
		coup.bots[2].cards = vec![Card::Ambassador, Card::Assassin];
		coup.playing_bots = vec![0, 1, 2, 3, 4];
		coup.turn = 0;

		coup.challenge_round_only(Action::Tax);

		assert_eq!(coup.bots[0].cards.len(), 2);
		assert_eq!(coup.bots[2].cards, vec![Card::Ambassador]);
	}

	#[test]
	fn test_counter_round_only_successful() {
		struct TestBot;
		impl BotInterface for TestBot {
			fn get_name(&self) -> String {
				String::from("TestBot")
			}
			fn on_counter(
				&self,
				_action: &Action,
				_by: String,
				_context: &Context,
			) -> Option<bool> {
				Some(true)
			}
		}
		struct ChallengeBot;
		impl BotInterface for ChallengeBot {
			fn get_name(&self) -> String {
				String::from("ChallengeBot")
			}
			fn on_challenge_counter_round(
				&self,
				_action: &Action,
				_by: String,
				_context: &Context,
			) -> bool {
				true
			}
		}

		let mut coup = Coup::new(vec![
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(TestBot),
			Box::new(ChallengeBot),
		]);
		coup.setup();
		coup.bots[0].cards = vec![Card::Assassin, Card::Captain];
		coup.bots[3].cards = vec![Card::Duke, Card::Assassin];
		coup.bots[4].cards = vec![Card::Captain, Card::Duke];
		coup.playing_bots = vec![0, 1, 2, 3, 4];
		coup.turn = 0;

		coup.counter_round_only();

		assert_eq!(coup.bots[0].cards, vec![Card::Assassin, Card::Captain]);
		assert_eq!(coup.bots[0].coins, 2);
		assert_eq!(coup.bots[3].cards, vec![Card::Duke, Card::Assassin]);
		assert_eq!(coup.bots[4].cards, vec![Card::Captain]);
	}

	#[test]
	fn test_counter_round_only_unsuccessful() {
		struct TestBot;
		impl BotInterface for TestBot {
			fn get_name(&self) -> String {
				String::from("TestBot")
			}
			fn on_counter(
				&self,
				_action: &Action,
				_by: String,
				_context: &Context,
			) -> Option<bool> {
				Some(true)
			}
		}
		struct ChallengeBot;
		impl BotInterface for ChallengeBot {
			fn get_name(&self) -> String {
				String::from("ChallengeBot")
			}
			fn on_challenge_counter_round(
				&self,
				_action: &Action,
				_by: String,
				_context: &Context,
			) -> bool {
				true
			}
		}

		let mut coup = Coup::new(vec![
			Box::new(StaticBot),
			Box::new(ChallengeBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(TestBot),
		]);
		coup.setup();
		coup.bots[0].cards = vec![Card::Assassin, Card::Captain];
		coup.bots[2].cards = vec![Card::Duke, Card::Duke];
		coup.bots[4].cards = vec![Card::Captain, Card::Assassin];
		coup.playing_bots = vec![0, 1, 2, 3, 4];
		coup.turn = 0;

		coup.counter_round_only();

		assert_eq!(coup.bots[0].cards, vec![Card::Assassin, Card::Captain]);
		assert_eq!(coup.bots[0].coins, 4);
		assert_eq!(coup.bots[2].cards, vec![Card::Duke, Card::Duke]);
		assert_eq!(coup.bots[4].cards, vec![Card::Captain]);
	}

	#[test]
	fn test_challenge_round_action_no_challenge() {
		struct TestBot {
			pub calls: std::cell::RefCell<Vec<String>>,
		}
		impl BotInterface for TestBot {
			fn get_name(&self) -> String {
				format!("TestBot{}", self.calls.borrow().join(","))
			}
			fn on_challenge_action_round(
				&self,
				_action: &Action,
				_by: String,
				_context: &Context,
			) -> bool {
				self.calls.borrow_mut().push(String::from("on_challenge_action_round"));
				false
			}
		}

		let mut coup = Coup::new(vec![
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
		]);
		coup.setup();

		coup.challenge_round(
			ChallengeRound::Action,
			&Action::Swapping,
			String::from("TestBot"),
		);

		assert_eq!(coup.bots[0].interface.get_name(), String::from("TestBot"));
		assert_eq!(
			coup.bots[1].interface.get_name(),
			String::from("TestBoton_challenge_action_round")
		);
		assert_eq!(
			coup.bots[2].interface.get_name(),
			String::from("TestBoton_challenge_action_round")
		);
		assert_eq!(
			coup.bots[3].interface.get_name(),
			String::from("TestBoton_challenge_action_round")
		);
		assert_eq!(
			coup.bots[4].interface.get_name(),
			String::from("TestBoton_challenge_action_round")
		);
		assert_eq!(coup.history, vec![]);

		coup.challenge_round(
			ChallengeRound::Action,
			&Action::Swapping,
			String::from("TestBot 3"),
		);

		assert_eq!(
			coup.bots[0].interface.get_name(),
			String::from("TestBoton_challenge_action_round")
		);
		assert_eq!(
			coup.bots[1].interface.get_name(),
			String::from(
				"TestBoton_challenge_action_round,on_challenge_action_round"
			)
		);
		assert_eq!(
			coup.bots[2].interface.get_name(),
			String::from("TestBoton_challenge_action_round")
		);
		assert_eq!(
			coup.bots[3].interface.get_name(),
			String::from(
				"TestBoton_challenge_action_round,on_challenge_action_round"
			)
		);
		assert_eq!(
			coup.bots[4].interface.get_name(),
			String::from(
				"TestBoton_challenge_action_round,on_challenge_action_round"
			)
		);
		assert_eq!(coup.history, vec![]);
	}

	#[test]
	fn test_challenge_round_action() {
		struct TestBot {
			pub calls: std::cell::RefCell<Vec<String>>,
		}
		impl BotInterface for TestBot {
			fn get_name(&self) -> String {
				format!("TestBot{}", self.calls.borrow().join(","))
			}
			fn on_challenge_action_round(
				&self,
				_action: &Action,
				_by: String,
				_context: &Context,
			) -> bool {
				self.calls.borrow_mut().push(String::from("on_challenge_action_round"));
				false
			}
		}
		pub struct ChallengeBot {
			pub calls: std::cell::RefCell<Vec<String>>,
		}
		impl BotInterface for ChallengeBot {
			fn get_name(&self) -> String {
				format!("ChallengeBot{}", self.calls.borrow().join(","))
			}
			fn on_challenge_action_round(
				&self,
				_action: &Action,
				_by: String,
				_context: &Context,
			) -> bool {
				self.calls.borrow_mut().push(String::from("on_challenge_action_round"));
				true
			}
		}

		let mut coup = Coup::new(vec![
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(ChallengeBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
		]);
		coup.setup();
		coup.playing_bots = vec![0, 1, 2, 3, 4];

		coup.challenge_round(
			ChallengeRound::Action,
			&Action::Swapping,
			String::from("TestBot"),
		);

		assert_eq!(coup.bots[0].interface.get_name(), String::from("TestBot"));
		assert_eq!(
			coup.bots[1].interface.get_name(),
			String::from("TestBoton_challenge_action_round")
		);
		assert_eq!(
			coup.bots[2].interface.get_name(),
			String::from("ChallengeBoton_challenge_action_round")
		);
		assert_eq!(coup.bots[3].interface.get_name(), String::from("TestBot"));
		assert_eq!(coup.bots[4].interface.get_name(), String::from("TestBot"));

		coup.challenge_round(
			ChallengeRound::Action,
			&Action::Swapping,
			String::from("TestBot 4"),
		);

		assert_eq!(
			coup.bots[0].interface.get_name(),
			String::from("TestBoton_challenge_action_round")
		);
		assert_eq!(
			coup.bots[1].interface.get_name(),
			String::from(
				"TestBoton_challenge_action_round,on_challenge_action_round"
			)
		);
		assert_eq!(
			coup.bots[2].interface.get_name(),
			String::from(
				"ChallengeBoton_challenge_action_round,on_challenge_action_round"
			)
		);
		assert_eq!(coup.bots[3].interface.get_name(), String::from("TestBot"));
		assert_eq!(coup.bots[4].interface.get_name(), String::from("TestBot"));
	}

	#[test]
	fn test_challenge_round_counter_no_challenge() {
		struct TestBot {
			pub calls: std::cell::RefCell<Vec<String>>,
		}
		impl BotInterface for TestBot {
			fn get_name(&self) -> String {
				format!("TestBot{}", self.calls.borrow().join(","))
			}
			fn on_challenge_counter_round(
				&self,
				_action: &Action,
				_by: String,
				_context: &Context,
			) -> bool {
				self
					.calls
					.borrow_mut()
					.push(String::from("on_challenge_counter_round"));
				false
			}
		}

		let mut coup = Coup::new(vec![
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
		]);
		coup.setup();

		coup.challenge_round(
			ChallengeRound::Counter,
			&Action::ForeignAid,
			String::from("TestBot 2"),
		);

		assert_eq!(
			coup.bots[0].interface.get_name(),
			String::from("TestBoton_challenge_counter_round")
		);
		assert_eq!(coup.bots[1].interface.get_name(), String::from("TestBot"));
		assert_eq!(
			coup.bots[2].interface.get_name(),
			String::from("TestBoton_challenge_counter_round")
		);
		assert_eq!(
			coup.bots[3].interface.get_name(),
			String::from("TestBoton_challenge_counter_round")
		);
		assert_eq!(
			coup.bots[4].interface.get_name(),
			String::from("TestBoton_challenge_counter_round")
		);
		assert_eq!(coup.history, vec![]);

		coup.challenge_round(
			ChallengeRound::Counter,
			&Action::ForeignAid,
			String::from("TestBot 5"),
		);

		assert_eq!(
			coup.bots[0].interface.get_name(),
			String::from(
				"TestBoton_challenge_counter_round,on_challenge_counter_round"
			)
		);
		assert_eq!(
			coup.bots[1].interface.get_name(),
			String::from("TestBoton_challenge_counter_round")
		);
		assert_eq!(
			coup.bots[2].interface.get_name(),
			String::from(
				"TestBoton_challenge_counter_round,on_challenge_counter_round"
			)
		);
		assert_eq!(
			coup.bots[3].interface.get_name(),
			String::from(
				"TestBoton_challenge_counter_round,on_challenge_counter_round"
			)
		);
		assert_eq!(
			coup.bots[4].interface.get_name(),
			String::from("TestBoton_challenge_counter_round")
		);
		assert_eq!(coup.history, vec![]);
	}

	#[test]
	fn test_challenge_round_counter() {
		struct TestBot {
			pub calls: std::cell::RefCell<Vec<String>>,
		}
		impl BotInterface for TestBot {
			fn get_name(&self) -> String {
				format!("TestBot{}", self.calls.borrow().join(","))
			}
			fn on_challenge_counter_round(
				&self,
				_action: &Action,
				_by: String,
				_context: &Context,
			) -> bool {
				self
					.calls
					.borrow_mut()
					.push(String::from("on_challenge_counter_round"));
				false
			}
		}
		pub struct ChallengeBot {
			pub calls: std::cell::RefCell<Vec<String>>,
		}
		impl BotInterface for ChallengeBot {
			fn get_name(&self) -> String {
				format!("ChallengeBot{}", self.calls.borrow().join(","))
			}
			fn on_challenge_counter_round(
				&self,
				_action: &Action,
				_by: String,
				_context: &Context,
			) -> bool {
				self
					.calls
					.borrow_mut()
					.push(String::from("on_challenge_counter_round"));
				true
			}
		}

		let mut coup = Coup::new(vec![
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(ChallengeBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
			Box::new(TestBot {
				calls: std::cell::RefCell::new(vec![]),
			}),
		]);
		coup.setup();
		coup.playing_bots = vec![0, 1, 2, 3, 4];

		coup.challenge_round(
			ChallengeRound::Counter,
			&Action::ForeignAid,
			String::from("TestBot 2"),
		);

		assert_eq!(
			coup.bots[0].interface.get_name(),
			String::from("TestBoton_challenge_counter_round")
		);
		assert_eq!(coup.bots[1].interface.get_name(), String::from("TestBot"));
		assert_eq!(
			coup.bots[2].interface.get_name(),
			String::from("TestBoton_challenge_counter_round")
		);
		assert_eq!(
			coup.bots[3].interface.get_name(),
			String::from("ChallengeBoton_challenge_counter_round")
		);
		assert_eq!(coup.bots[4].interface.get_name(), String::from("TestBot"));

		coup.challenge_round(
			ChallengeRound::Counter,
			&Action::ForeignAid,
			String::from("TestBot 4"),
		);

		assert_eq!(
			coup.bots[0].interface.get_name(),
			String::from(
				"TestBoton_challenge_counter_round,on_challenge_counter_round"
			)
		);
		assert_eq!(
			coup.bots[1].interface.get_name(),
			String::from("TestBoton_challenge_counter_round")
		);
		assert_eq!(
			coup.bots[2].interface.get_name(),
			String::from(
				"TestBoton_challenge_counter_round,on_challenge_counter_round"
			)
		);
		assert_eq!(
			coup.bots[3].interface.get_name(),
			String::from(
				"ChallengeBoton_challenge_counter_round,on_challenge_counter_round"
			)
		);
		assert_eq!(coup.bots[4].interface.get_name(), String::from("TestBot"));
	}

	#[test]
	fn test_resolve_challenge_successful() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		// Assassination
		coup.bots[0].cards = vec![Card::Duke, Card::Captain];
		coup.bots[1].cards = vec![Card::Ambassador, Card::Ambassador];

		let result = coup.resolve_challenge(
			Action::Assassination(String::from("StaticBot 2")),
			String::from("StaticBot"),
			String::from("StaticBot 2"),
		);

		assert_eq!(result, true);
		assert_eq!(coup.bots[0].cards, vec![Card::Duke]);
		assert_eq!(coup.bots[1].cards, vec![Card::Ambassador, Card::Ambassador]);
		assert_eq!(
			coup.history,
			vec![History::ChallengeAssassin {
				by: String::from("StaticBot 2"),
				target: String::from("StaticBot")
			}]
		);
		coup.history = vec![];

		// Swapping
		coup.bots[0].cards = vec![Card::Duke, Card::Captain];
		coup.bots[1].cards = vec![Card::Ambassador, Card::Ambassador];

		let result = coup.resolve_challenge(
			Action::Swapping,
			String::from("StaticBot"),
			String::from("StaticBot 2"),
		);

		assert_eq!(result, true);
		assert_eq!(coup.bots[0].cards, vec![Card::Duke]);
		assert_eq!(coup.bots[1].cards, vec![Card::Ambassador, Card::Ambassador]);
		assert_eq!(
			coup.history,
			vec![History::ChallengeAmbassador {
				by: String::from("StaticBot 2"),
				target: String::from("StaticBot")
			}]
		);
		coup.history = vec![];

		// Stealing
		coup.bots[0].cards = vec![Card::Duke, Card::Assassin];
		coup.bots[1].cards = vec![Card::Ambassador, Card::Ambassador];

		let result = coup.resolve_challenge(
			Action::Stealing(String::from("StaticBot 2")),
			String::from("StaticBot"),
			String::from("StaticBot 2"),
		);

		assert_eq!(result, true);
		assert_eq!(coup.bots[0].cards, vec![Card::Duke]);
		assert_eq!(coup.bots[1].cards, vec![Card::Ambassador, Card::Ambassador]);
		assert_eq!(
			coup.history,
			vec![History::ChallengeCaptain {
				by: String::from("StaticBot 2"),
				target: String::from("StaticBot")
			}]
		);
		coup.history = vec![];

		// Tax
		coup.bots[0].cards = vec![Card::Captain, Card::Captain];
		coup.bots[1].cards = vec![Card::Ambassador, Card::Ambassador];

		let result = coup.resolve_challenge(
			Action::Tax,
			String::from("StaticBot"),
			String::from("StaticBot 2"),
		);

		assert_eq!(result, true);
		assert_eq!(coup.bots[0].cards, vec![Card::Captain]);
		assert_eq!(coup.bots[1].cards, vec![Card::Ambassador, Card::Ambassador]);
		assert_eq!(
			coup.history,
			vec![History::ChallengeDuke {
				by: String::from("StaticBot 2"),
				target: String::from("StaticBot")
			}]
		);
	}

	#[test]
	fn test_resolve_challenge_unsuccessful() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		// Assassination
		coup.bots[0].cards = vec![Card::Assassin, Card::Captain];
		coup.bots[1].cards = vec![Card::Ambassador, Card::Ambassador];

		let result = coup.resolve_challenge(
			Action::Assassination(String::from("StaticBot 2")),
			String::from("StaticBot"),
			String::from("StaticBot 2"),
		);

		assert_eq!(result, false);
		assert_eq!(coup.bots[0].cards, vec![Card::Assassin, Card::Captain]);
		assert_eq!(coup.bots[1].cards, vec![Card::Ambassador]);
		assert_eq!(
			coup.history,
			vec![History::ChallengeAssassin {
				by: String::from("StaticBot 2"),
				target: String::from("StaticBot")
			}]
		);
		coup.history = vec![];

		// Swapping
		coup.bots[0].cards = vec![Card::Assassin, Card::Ambassador];
		coup.bots[1].cards = vec![Card::Ambassador, Card::Ambassador];

		let result = coup.resolve_challenge(
			Action::Swapping,
			String::from("StaticBot"),
			String::from("StaticBot 2"),
		);

		assert_eq!(result, false);
		assert_eq!(coup.bots[0].cards, vec![Card::Assassin, Card::Ambassador]);
		assert_eq!(coup.bots[1].cards, vec![Card::Ambassador]);
		assert_eq!(
			coup.history,
			vec![History::ChallengeAmbassador {
				by: String::from("StaticBot 2"),
				target: String::from("StaticBot")
			}]
		);
		coup.history = vec![];

		// Stealing
		coup.bots[0].cards = vec![Card::Assassin, Card::Captain];
		coup.bots[1].cards = vec![Card::Ambassador, Card::Ambassador];

		let result = coup.resolve_challenge(
			Action::Stealing(String::from("StaticBot 2")),
			String::from("StaticBot"),
			String::from("StaticBot 2"),
		);

		assert_eq!(result, false);
		assert_eq!(coup.bots[0].cards, vec![Card::Assassin, Card::Captain]);
		assert_eq!(coup.bots[1].cards, vec![Card::Ambassador]);
		assert_eq!(
			coup.history,
			vec![History::ChallengeCaptain {
				by: String::from("StaticBot 2"),
				target: String::from("StaticBot")
			}]
		);
		coup.history = vec![];

		// Tax
		coup.bots[0].cards = vec![Card::Duke, Card::Captain];
		coup.bots[1].cards = vec![Card::Ambassador, Card::Ambassador];

		let result = coup.resolve_challenge(
			Action::Tax,
			String::from("StaticBot"),
			String::from("StaticBot 2"),
		);

		assert_eq!(result, false);
		assert_eq!(coup.bots[0].cards, vec![Card::Duke, Card::Captain]);
		assert_eq!(coup.bots[1].cards, vec![Card::Ambassador]);
		assert_eq!(
			coup.history,
			vec![History::ChallengeDuke {
				by: String::from("StaticBot 2"),
				target: String::from("StaticBot")
			}]
		);
		coup.history = vec![];
	}

	#[test]
	fn test_resolve_counter_challenge_successful() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		// Assassination
		coup.bots[0].cards = vec![Card::Duke, Card::Captain];
		coup.bots[1].cards = vec![Card::Ambassador, Card::Ambassador];

		let result = coup.resolve_counter_challenge(
			Counter::Assassination,
			String::from("StaticBot"),
			String::from("StaticBot 2"),
		);

		assert_eq!(result, true);
		assert_eq!(coup.bots[0].cards, vec![Card::Duke]);
		assert_eq!(coup.bots[1].cards, vec![Card::Ambassador, Card::Ambassador]);
		assert_eq!(
			coup.history,
			vec![History::CounterChallengeContessa {
				by: String::from("StaticBot 2"),
				target: String::from("StaticBot")
			}]
		);
		coup.history = vec![];

		// Foreign Aid
		coup.bots[0].cards = vec![Card::Assassin, Card::Captain];
		coup.bots[1].cards = vec![Card::Ambassador, Card::Ambassador];

		let result = coup.resolve_counter_challenge(
			Counter::ForeignAid,
			String::from("StaticBot"),
			String::from("StaticBot 2"),
		);

		assert_eq!(result, true);
		assert_eq!(coup.bots[0].cards, vec![Card::Assassin]);
		assert_eq!(coup.bots[1].cards, vec![Card::Ambassador, Card::Ambassador]);
		assert_eq!(
			coup.history,
			vec![History::CounterChallengeDuke {
				by: String::from("StaticBot 2"),
				target: String::from("StaticBot")
			}]
		);
		coup.history = vec![];

		// Stealing
		coup.bots[0].cards = vec![Card::Assassin, Card::Contessa];
		coup.bots[1].cards = vec![Card::Ambassador, Card::Ambassador];

		let result = coup.resolve_counter_challenge(
			Counter::Stealing,
			String::from("StaticBot"),
			String::from("StaticBot 2"),
		);

		assert_eq!(result, true);
		assert_eq!(coup.bots[0].cards, vec![Card::Assassin]);
		assert_eq!(coup.bots[1].cards, vec![Card::Ambassador, Card::Ambassador]);
		assert_eq!(
			coup.history,
			vec![History::CounterChallengeCaptainAmbassedor {
				by: String::from("StaticBot 2"),
				target: String::from("StaticBot")
			}]
		);
	}

	#[test]
	fn test_resolve_counter_challenge_unsuccessful() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		// Assassination
		coup.bots[0].cards = vec![Card::Assassin, Card::Contessa];
		coup.bots[1].cards = vec![Card::Ambassador, Card::Ambassador];

		let result = coup.resolve_counter_challenge(
			Counter::Assassination,
			String::from("StaticBot"),
			String::from("StaticBot 2"),
		);

		assert_eq!(result, false);
		assert_eq!(coup.bots[0].cards, vec![Card::Assassin, Card::Contessa]);
		assert_eq!(coup.bots[1].cards, vec![Card::Ambassador]);
		assert_eq!(
			coup.history,
			vec![History::CounterChallengeContessa {
				by: String::from("StaticBot 2"),
				target: String::from("StaticBot")
			}]
		);
		coup.history = vec![];

		// Foreign Aid
		coup.bots[0].cards = vec![Card::Duke, Card::Contessa];
		coup.bots[1].cards = vec![Card::Ambassador, Card::Ambassador];

		let result = coup.resolve_counter_challenge(
			Counter::ForeignAid,
			String::from("StaticBot"),
			String::from("StaticBot 2"),
		);

		assert_eq!(result, false);
		assert_eq!(coup.bots[0].cards, vec![Card::Duke, Card::Contessa]);
		assert_eq!(coup.bots[1].cards, vec![Card::Ambassador]);
		assert_eq!(
			coup.history,
			vec![History::CounterChallengeDuke {
				by: String::from("StaticBot 2"),
				target: String::from("StaticBot")
			}]
		);
		coup.history = vec![];

		// Stealing with Captain
		coup.bots[0].cards = vec![Card::Duke, Card::Captain];
		coup.bots[1].cards = vec![Card::Ambassador, Card::Ambassador];

		let result = coup.resolve_counter_challenge(
			Counter::Stealing,
			String::from("StaticBot"),
			String::from("StaticBot 2"),
		);

		assert_eq!(result, false);
		assert_eq!(coup.bots[0].cards, vec![Card::Duke, Card::Captain]);
		assert_eq!(coup.bots[1].cards, vec![Card::Ambassador]);
		assert_eq!(
			coup.history,
			vec![History::CounterChallengeCaptainAmbassedor {
				by: String::from("StaticBot 2"),
				target: String::from("StaticBot")
			}]
		);
		coup.history = vec![];

		// Stealing with Ambassador
		coup.bots[0].cards = vec![Card::Duke, Card::Ambassador];
		coup.bots[1].cards = vec![Card::Ambassador, Card::Ambassador];

		let result = coup.resolve_counter_challenge(
			Counter::Stealing,
			String::from("StaticBot"),
			String::from("StaticBot 2"),
		);

		assert_eq!(result, false);
		assert_eq!(coup.bots[0].cards, vec![Card::Duke, Card::Ambassador]);
		assert_eq!(coup.bots[1].cards, vec![Card::Ambassador]);
		assert_eq!(
			coup.history,
			vec![History::CounterChallengeCaptainAmbassedor {
				by: String::from("StaticBot 2"),
				target: String::from("StaticBot")
			}]
		);
	}

	// TODO: test_display_score

	#[test]
	fn test_format_number_with_separator() {
		assert_eq!(Coup::format_number_with_separator(0), String::from("0"));
		assert_eq!(Coup::format_number_with_separator(00000), String::from("0"));
		assert_eq!(Coup::format_number_with_separator(1), String::from("1"));
		assert_eq!(Coup::format_number_with_separator(99), String::from("99"));
		assert_eq!(Coup::format_number_with_separator(999), String::from("999"));
		assert_eq!(Coup::format_number_with_separator(1234), String::from("1,234"));
		assert_eq!(
			Coup::format_number_with_separator(9876543210),
			String::from("9,876,543,210")
		);
	}

	// TODO: test_looping

	// *******************************| Actions |****************************** //
	#[test]
	fn test_action_assassination() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		coup.bots[0].cards = vec![Card::Ambassador, Card::Duke];
		coup.bots[1].cards = vec![Card::Assassin, Card::Captain];
		coup.playing_bots = vec![0, 1];
		coup.bots[0].coins = 4;
		coup.deck = vec![Card::Ambassador, Card::Captain];

		coup.action_assassination(String::from("StaticBot 2"));

		assert_eq!(coup.bots[0].cards, vec![Card::Ambassador, Card::Duke]);
		assert_eq!(coup.bots[1].cards, vec![Card::Assassin]);
		assert_eq!(coup.bots[0].coins, 1);
		assert_eq!(coup.deck, vec![Card::Ambassador, Card::Captain]);
		assert_eq!(coup.discard_pile, vec![Card::Captain]);
	}

	#[test]
	fn test_action_assassination_unknown_bot() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		coup.bots[0].cards = vec![Card::Ambassador, Card::Duke];
		coup.bots[1].cards = vec![Card::Assassin, Card::Captain];
		coup.playing_bots = vec![0, 1];
		coup.bots[0].coins = 4;
		coup.deck = vec![Card::Ambassador, Card::Captain];

		coup.action_assassination(String::from("Unknown bot"));

		assert_eq!(coup.bots[0].cards, vec![Card::Ambassador]);
		assert_eq!(coup.bots[1].cards, vec![Card::Assassin, Card::Captain]);
		assert_eq!(coup.bots[0].coins, 4);
		assert_eq!(coup.deck, vec![Card::Ambassador, Card::Captain]);
		assert_eq!(coup.discard_pile, vec![Card::Duke]);
	}

	#[test]
	fn test_action_assassination_insufficient_funds() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		coup.bots[0].cards = vec![Card::Ambassador, Card::Duke];
		coup.bots[1].cards = vec![Card::Assassin, Card::Captain];
		coup.playing_bots = vec![0, 1];
		coup.bots[0].coins = 2;
		coup.deck = vec![Card::Ambassador, Card::Captain];

		coup.action_assassination(String::from("StaticBot 2"));

		assert_eq!(coup.bots[0].cards, vec![Card::Ambassador]);
		assert_eq!(coup.bots[1].cards, vec![Card::Assassin, Card::Captain]);
		assert_eq!(coup.bots[0].coins, 2);
		assert_eq!(coup.deck, vec![Card::Ambassador, Card::Captain]);
		assert_eq!(coup.discard_pile, vec![Card::Duke]);
	}

	#[test]
	fn test_action_couping() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		coup.bots[0].cards = vec![Card::Ambassador, Card::Duke];
		coup.bots[1].cards = vec![Card::Assassin, Card::Captain];
		coup.playing_bots = vec![0, 1];
		coup.bots[0].coins = 8;
		coup.deck = vec![Card::Ambassador, Card::Captain];

		coup.action_couping(String::from("StaticBot 2"));

		assert_eq!(coup.bots[0].cards, vec![Card::Ambassador, Card::Duke]);
		assert_eq!(coup.bots[1].cards, vec![Card::Assassin]);
		assert_eq!(coup.bots[0].coins, 1);
		assert_eq!(coup.deck, vec![Card::Ambassador, Card::Captain]);
		assert_eq!(coup.discard_pile, vec![Card::Captain]);
	}

	#[test]
	fn test_action_couping_unknown_bot() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		coup.bots[0].cards = vec![Card::Ambassador, Card::Duke];
		coup.bots[1].cards = vec![Card::Assassin, Card::Captain];
		coup.playing_bots = vec![0, 1];
		coup.bots[0].coins = 8;
		coup.deck = vec![Card::Ambassador, Card::Captain];

		coup.action_couping(String::from("Unknown bot"));

		assert_eq!(coup.bots[0].cards, vec![Card::Ambassador]);
		assert_eq!(coup.bots[1].cards, vec![Card::Assassin, Card::Captain]);
		assert_eq!(coup.bots[0].coins, 8);
		assert_eq!(coup.deck, vec![Card::Ambassador, Card::Captain]);
		assert_eq!(coup.discard_pile, vec![Card::Duke]);
	}

	#[test]
	fn test_action_couping_insufficient_funds() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		coup.bots[0].cards = vec![Card::Ambassador, Card::Duke];
		coup.bots[1].cards = vec![Card::Assassin, Card::Captain];
		coup.playing_bots = vec![0, 1];
		coup.bots[0].coins = 6;
		coup.deck = vec![Card::Ambassador, Card::Captain];

		coup.action_couping(String::from("StaticBot 2"));

		assert_eq!(coup.bots[0].cards, vec![Card::Ambassador]);
		assert_eq!(coup.bots[1].cards, vec![Card::Assassin, Card::Captain]);
		assert_eq!(coup.bots[0].coins, 6);
		assert_eq!(coup.deck, vec![Card::Ambassador, Card::Captain]);
		assert_eq!(coup.discard_pile, vec![Card::Duke]);
	}

	#[test]
	fn test_action_foraign_aid() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();
		coup.playing_bots = vec![0, 1];

		coup.action_foraign_aid();

		assert_eq!(coup.bots[0].coins, 4);
		assert_eq!(coup.bots[1].coins, 2);
	}

	#[test]
	fn test_action_swapping() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();

		coup.bots[0].cards = vec![Card::Ambassador, Card::Duke];
		coup.bots[1].cards = vec![Card::Assassin, Card::Captain];
		coup.playing_bots = vec![0, 1];
		coup.deck = vec![Card::Captain, Card::Assassin];

		coup.action_swapping();

		assert_eq!(coup.bots[0].cards, vec![Card::Ambassador, Card::Duke]);
		assert_eq!(coup.bots[1].cards, vec![Card::Assassin, Card::Captain]);
		assert_eq!(coup.deck.len(), 2);
	}

	#[test]
	fn test_action_swapping_custom_bot() {
		struct TestBot;
		impl BotInterface for TestBot {
			fn get_name(&self) -> String {
				String::from("TestBot")
			}
			fn on_swapping_cards(
				&self,
				new_cards: [Card; 2],
				context: &Context,
			) -> [Card; 2] {
				[new_cards[1], context.cards[1]]
			}
		}

		let mut coup = Coup::new(vec![Box::new(TestBot), Box::new(StaticBot)]);
		coup.setup();

		coup.bots[0].cards = vec![Card::Ambassador, Card::Duke];
		coup.bots[1].cards = vec![Card::Assassin, Card::Captain];
		coup.playing_bots = vec![0, 1];
		coup.deck = vec![Card::Assassin, Card::Captain];

		coup.action_swapping();

		assert_eq!(coup.bots[0].cards, vec![Card::Ambassador, Card::Captain]);
		assert_eq!(coup.bots[1].cards, vec![Card::Assassin, Card::Captain]);
		assert_eq!(coup.deck.len(), 2);
	}

	#[test]
	fn test_action_swapping_faulty_bot() {
		struct TestBot;
		impl BotInterface for TestBot {
			fn get_name(&self) -> String {
				String::from("TestBot")
			}
			fn on_swapping_cards(
				&self,
				_new_cards: [Card; 2],
				_context: &Context,
			) -> [Card; 2] {
				[Card::Assassin, Card::Duke]
			}
		}

		let mut coup = Coup::new(vec![Box::new(TestBot), Box::new(StaticBot)]);
		coup.setup();

		coup.bots[0].cards = vec![Card::Ambassador, Card::Duke];
		coup.bots[1].cards = vec![Card::Assassin, Card::Captain];
		coup.playing_bots = vec![0, 1];
		coup.deck = vec![Card::Ambassador, Card::Captain];

		coup.action_swapping();

		assert_eq!(coup.bots[0].cards, vec![Card::Ambassador]);
		assert_eq!(coup.bots[1].cards, vec![Card::Assassin, Card::Captain]);
		assert_eq!(coup.deck.len(), 0);
	}

	#[test]
	fn test_action_income() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();
		coup.playing_bots = vec![0, 1];

		coup.action_income();

		assert_eq!(coup.bots[0].coins, 3);
		assert_eq!(coup.bots[1].coins, 2);
	}

	#[test]
	fn test_action_stealing() {
		let mut coup = Coup::new(vec![
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
		]);
		coup.setup();
		coup.playing_bots = vec![0, 1, 2, 3];

		coup.action_stealing(String::from("StaticBot 3"));

		assert_eq!(coup.bots[0].coins, 4);
		assert_eq!(coup.bots[1].coins, 2);
		assert_eq!(coup.bots[2].coins, 0);
		assert_eq!(coup.bots[3].coins, 2);
	}

	#[test]
	fn test_action_stealing_min() {
		let mut coup = Coup::new(vec![
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
		]);
		coup.setup();
		coup.playing_bots = vec![0, 1, 2, 3];
		coup.bots[2].coins = 1;

		coup.action_stealing(String::from("StaticBot 3"));

		assert_eq!(coup.bots[0].coins, 3);
		assert_eq!(coup.bots[1].coins, 2);
		assert_eq!(coup.bots[2].coins, 0);
		assert_eq!(coup.bots[3].coins, 2);
	}

	#[test]
	fn test_action_stealing_max() {
		let mut coup = Coup::new(vec![
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
			Box::new(StaticBot),
		]);
		coup.setup();
		coup.playing_bots = vec![0, 1, 2, 3];
		coup.bots[2].coins = 5;

		coup.action_stealing(String::from("StaticBot 3"));

		assert_eq!(coup.bots[0].coins, 4);
		assert_eq!(coup.bots[1].coins, 2);
		assert_eq!(coup.bots[2].coins, 3);
		assert_eq!(coup.bots[3].coins, 2);
	}

	#[test]
	fn test_action_tax() {
		let mut coup = Coup::new(vec![Box::new(StaticBot), Box::new(StaticBot)]);
		coup.setup();
		coup.playing_bots = vec![0, 1];

		coup.action_tax();

		assert_eq!(coup.bots[0].coins, 5);
		assert_eq!(coup.bots[1].coins, 2);
	}
}
