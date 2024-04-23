use std::fmt;

use rand::{seq::SliceRandom, thread_rng};

pub mod bot;
pub mod bots;

use crate::bot::{BotInterface, Context, OtherBot};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Card {
	Ambassador,
	Assassin,
	Captain,
	Contessa,
	Duke,
}

/// Actions that can we taken with a [Card] you have
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
	/// Take this action with your [Card::Assassin]
	Assassination(String),
	/// This standard action can be taken at any time as long as you have at least 7 coin
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

/// Counters are played if something happens that can be countered with a [Card] you have
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Counter {
	/// Block an assassination with your [Card::Contessa]
	Assassination,
	/// Block foreign aid with your [Card::Duke]
	ForeignAid,
	/// Block stealing with your [Card::Captain]
	StealingCaptain,
	/// Block stealing with your [Card::Ambassador]
	StealingAmbassador,
	/// block tax with your [Card::Duke]
	Tax,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum History {
	ActionAssassination { by: String, target: String },
	ActionCoup { by: String, target: String },
	ActionForeignAid { by: String },
	ActionSwapping { by: String },
	ActionIncome { by: String },
	ActionStealing { by: String, target: String },
	ActionTax { by: String, target: String },

	ChallengeAssassin { by: String, target: String },
	ChallengeAmbassador { by: String, target: String },
	ChallengeCaptain { by: String, target: String },
	ChallengeDuke { by: String, target: String },

	CounterAssassination { by: String, target: String },
	CounterForeignAid { by: String, target: String },
	CounterStealing { by: String, target: String },
	CounterTax { by: String, target: String },
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
	pub moves: usize,
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
			moves: 0,
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

		self.turn = 0;
		self.moves = 0;

		// TODO: add cfonts logo here

		// Let's play
		self.game_loop();
	}

	#[allow(clippy::borrowed_box)]
	fn get_bot_by_name(&self, name: String) -> &Box<dyn BotInterface> {
		self.bots.iter().find(|bot| bot.get_name() == name).unwrap()
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

	fn penalize_bot(&mut self, name: String, reason: &str, context: Context) {
		self.bots.iter_mut().for_each(|bot| {
			if bot.get_name() == name {
				println!(
					"🚨  {} is being penalized because \x1b[33m{}\x1b[39m",
					bot, reason
				);
				let lost_card = bot.on_card_loss(context.clone());

				bot.set_cards(
					bot
						.get_cards()
						.into_iter()
						.filter(|card| lost_card != *card)
						.collect(),
				);
				println!(
					"{}  {} has lost the \x1b[33m{:?}\x1b[39m",
					if bot.get_cards().is_empty() {
						"☠️ "
					} else {
						"💔"
					},
					bot,
					lost_card
				);
			}
		});
	}

	fn target_not_found(&self, target: String) -> bool {
		self.bots.iter().filter(|bot| bot.get_name() == target).count() == 0
	}

	/// Play the game with the round that has been setup
	pub fn game_loop(&mut self) {
		while self.has_not_ended() {
			self.moves += 1;
			let playing_bot = &self.bots[self.playing_bots[self.turn]];
			let playing_bot_name = playing_bot.get_name();
			let playing_bot_coins = playing_bot.get_coins();
			let _playing_bot_cards = playing_bot.get_cards();

			let context = Context {
				other_bots: &self.get_other_bots(),
				discard_pile: &self.discard_pile.clone(),
				history: &self.history.clone(),
				score: &self.score.clone(),
			};

			// If you have 10 or more coins you must coup
			let action = if self.bots[self.playing_bots[self.turn]].get_coins() >= 10
			{
				let target =
					self.bots[self.playing_bots[self.turn]].on_auto_coup(context.clone());
				Action::Coup(target)
			} else {
				self.bots[self.playing_bots[self.turn]].on_turn(context.clone())
			};

			match action {
				Action::Assassination(_target) => {
					// challenge round (aka: Do I believe you have this card?) bot.on_challenge_action_round
					//   if challenge successful
					//     self penalty
					//   if challenge not successful
					//     challenger penalty

					// counter round (aka: do I have a card to block this action?) bot.on_counter
					//   if counter
					//     challenge round
					//       if challenge successful
					//         counter penalty
					//         self.action
					//       if challenge not successful
					//         challenger penalty
					//   if no counter
					//     self.action
					todo!()
				},
				Action::Coup(ref target) => {
					self.action_couping(
						target.clone(),
						playing_bot_coins,
						playing_bot_name,
						context,
					);
				},
				Action::ForeignAid => {
					// counter round (aka: do I have a card to block this action?) bot.on_counter
					//   if counter
					//     challenge round
					//       if challenge successful
					//         counter penalty
					//         self.action
					//       if challenge not successful
					//         challenger penalty
					//   if no counter
					//     self.action
					todo!()
				},
				Action::Swapping => {
					// challenge round (aka: Do I believe you have this card?) bot.on_challenge_action_round
					//   if challenge successful
					//     self penalty
					//   if challenge not successful
					//     challenger penalty
					//     self.action
					todo!()
				},
				Action::Income => {
					self.action_income(playing_bot_coins, playing_bot_name)
				},
				Action::Stealing(_target) => {
					// challenge round (aka: Do I believe you have this card?) bot.on_challenge_action_round
					//   if challenge successful
					//     self penalty
					//   if challenge not successful
					//     challenger penalty

					// counter round (aka: do I have a card to block this action?) bot.on_counter
					//   if counter
					//     challenge round
					//       if challenge successful
					//         counter penalty
					//         self.action
					//       if challenge not successful
					//         challenger penalty
					//   if no counter
					//     self.action
					todo!()
				},
				Action::Tax => {
					// challenge round (aka: Do I believe you have this card?) bot.on_challenge_action_round
					//   if challenge successful
					//     self penalty
					//   if challenge not successful
					//     challenger penalty

					// counter round (aka: do I have a card to block this action?) bot.on_counter
					//   if counter
					//     challenge round
					//       if challenge successful
					//         counter penalty
					//         self.action
					//       if challenge not successful
					//         challenger penalty
					//   if no counter
					//     self.action
					todo!()
				},
			}

			// Let's filter out all dead bots
			self.playing_bots = self
				.playing_bots
				.iter()
				.filter(|bot_index| !self.bots[**bot_index].get_cards().is_empty())
				.copied()
				.collect::<Vec<usize>>();

			// We move to the next turn
			self.turn = if self.turn >= self.playing_bots.len() - 1 {
				0
			} else {
				self.turn + 1
			};
		}

		let winner = &self.bots[self.playing_bots[0]];
		println!("\nThe winner is {} in {} moves", winner, self.moves);
	}

	fn action_income(&mut self, playing_bot_coins: u8, playing_bot_name: String) {
		// Adding the coin to the bot
		self.bots[self.playing_bots[self.turn]].set_coins(playing_bot_coins + 1);

		// Logging
		self.history.push(History::ActionIncome {
			by: playing_bot_name.clone(),
		});
		println!(
			"🃏  {} takes \x1b[33ma coin\x1b[39m",
			self.bots[self.playing_bots[self.turn]]
		);
	}

	fn action_couping(
		&mut self,
		target: String,
		playing_bot_coins: u8,
		playing_bot_name: String,
		context: Context,
	) {
		if playing_bot_coins < 7 {
			self.penalize_bot(
				playing_bot_name.clone(),
				"it tried to coup with insufficient funds",
				context,
			);
		} else if self.target_not_found(target.clone()) {
			self.penalize_bot(
				playing_bot_name.clone(),
				"it tried to coup an unknown bot",
				context,
			);
		} else {
			// Paying the fee
			self.bots[self.playing_bots[self.turn]].set_coins(playing_bot_coins - 7);

			// Taking a card from the target bot
			let target_bot = self.get_bot_by_name(target.clone());
			let target_bot_name = target_bot.get_name();

			let lost_card = target_bot.on_card_loss(context.clone());
			if !target_bot.get_cards().contains(&lost_card) {
				self.penalize_bot(
					playing_bot_name.clone(),
					"it tried discard a card it didn't have",
					context,
				);
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

			// Logging
			self.history.push(History::ActionCoup {
				by: playing_bot_name.clone(),
				target: target_bot_name,
			});
			println!(
				"🃏  {} coups {}",
				self.bots[self.playing_bots[self.turn]],
				self.get_bot_by_name(target.clone())
			);
		}
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
