use std::fmt;

use rand::{seq::SliceRandom, thread_rng};

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
	/// - [Counter::Assassination] – Block an assassination attempt against yourself.
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
	ActionTax { by: String, target: String },

	ChallengeAssassin { by: String, target: String },
	ChallengeAmbassador { by: String, target: String },
	ChallengeCaptain { by: String, target: String },
	ChallengeDuke { by: String, target: String },

	CounterAssassination { by: String, target: String },
	CounterForeignAid { by: String, target: String },
	CounterStealing { by: String, target: String },
}

/// The score of the game for all bots
pub type Score = Vec<(String, u64)>;

/// The Coup game engine
pub struct Coup {
	bots: Vec<Box<dyn BotInterface>>,
	playing_bots: Vec<usize>,
	deck: Vec<Card>,
	discard_pile: Vec<Card>,
	history: Vec<History>,
	score: Score,
	turn: usize,
	moves: usize,
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
		for index in 0..std::cmp::min(self.bots.len(), 6) {
			self.playing_bots.push(index);
		}

		// Shuffle all bots each round and limit them to the max players per game
		self.playing_bots.shuffle(&mut thread_rng());
		self.playing_bots.truncate(6);

		// Give all playing bots cards and coins
		for bot in self.playing_bots.iter() {
			let new_cards = vec![deck.pop().unwrap(), deck.pop().unwrap()];
			self.bots[*bot].set_cards(new_cards);
			self.bots[*bot].set_coins(2);
		}
		self.deck = deck;

		self.discard_pile = vec![];
		self.turn = 0;
		self.moves = 0;
	}

	fn log(message: std::fmt::Arguments) {
		if std::env::var("NOLOG").is_err() {
			println!("{:?}", message);
		}
	}

	/// Playing a game which means we setup the table, give each bots their cards
	/// and coins and start the game loop
	pub fn play(&mut self) {
		self.setup();
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

	fn card_loss(&mut self, name: String, context: &Context) {
		self.bots.iter_mut().for_each(|bot| {
			if bot.get_name() == name {
				let lost_card = bot.on_card_loss(context);
				if !bot.get_cards().contains(&lost_card) {
					Self::log(format_args!("🚨  {} is being penalized because \x1b[33mit discarded a card it didn't have\x1b[39m", bot));
					let mut cards = bot.get_cards();

					let card = cards.pop().unwrap();
					let mut lost_cards = format!("{:?}", card);
					self.discard_pile.push(card);

					if !cards.is_empty() {
						let card = cards.pop().unwrap();
						lost_cards =
							format!("{} and {:?}", lost_cards, card);
						self.discard_pile.push(card);
					}

					bot.set_cards(vec![]);
					Self::log(format_args!("☠️   {} has lost the \x1b[33m{:?}\x1b[39m", bot, lost_cards));
				} else {
					bot.set_cards(
						bot
							.get_cards()
							.into_iter()
							.filter(|card| lost_card != *card)
							.collect(),
					);
					self.discard_pile.push(lost_card);

					Self::log(format_args!(
						"{}  {} has lost the \x1b[33m{:?}\x1b[39m",
						if bot.get_cards().is_empty() {
							"☠️ "
						} else {
							"💔"
						},
						bot,
						lost_card
					));
				}
			}
		});
	}

	fn penalize_bot(&mut self, name: String, reason: &str, context: &Context) {
		Self::log(format_args!(
			"🚨  {} is being penalized because \x1b[33m{}\x1b[39m",
			self.get_bot_by_name(name.clone()),
			reason
		));
		self.card_loss(name, context);
	}

	fn target_not_found(&self, target: String) -> bool {
		self.bots.iter().filter(|bot| bot.get_name() == target).count() == 0
	}

	fn game_loop(&mut self) {
		while self.playing_bots.len() > 1 {
			self.moves += 1;
			let playing_bot = &self.bots[self.playing_bots[self.turn]];
			let playing_bot_name = playing_bot.get_name();
			let playing_bot_coins = playing_bot.get_coins();
			let _playing_bot_cards = playing_bot.get_cards();

			let context = Context {
				other_bots: self.get_other_bots(),
				discard_pile: self.discard_pile.clone(),
				history: self.history.clone(),
				score: self.score.clone(),
			};

			// If you have 10 or more coins you must coup
			let action = if self.bots[self.playing_bots[self.turn]].get_coins() >= 10
			{
				let target =
					self.bots[self.playing_bots[self.turn]].on_auto_coup(&context);
				Action::Coup(target)
			} else {
				self.bots[self.playing_bots[self.turn]].on_turn(&context)
			};

			match action {
				Action::Assassination(target_name) => {
					self.challenge_and_counter_round(
						Action::Assassination(target_name.clone()),
						playing_bot_name,
						target_name,
						&context,
					);
				},
				Action::Coup(ref target) => {
					self.action_couping(
						target.clone(),
						playing_bot_coins,
						playing_bot_name,
						&context,
					);
				},
				Action::ForeignAid => {
					// counter round only
					todo!()
				},
				Action::Swapping => {
					// challenge round only
					todo!()
				},
				Action::Income => {
					self.action_income(playing_bot_coins, playing_bot_name)
				},
				Action::Stealing(target_name) => {
					self.challenge_and_counter_round(
						Action::Stealing(target_name.clone()),
						playing_bot_name,
						target_name,
						&context,
					);
				},
				Action::Tax => {
					// challenge round only
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
		Self::log(format_args!(
			"\n🎉🎉🎉 The winner is {} \x1b[90min {} moves\x1b[39m\n",
			winner, self.moves
		));
	}

	fn challenge_and_counter_round(
		&mut self,
		action: Action,
		playing_bot_name: String,
		target_name: String,
		context: &Context,
	) {
		// THE CHALLENGE ROUND
		// Does anyone want to challenge this action?
		if let Some(challenger) = self.challenge_round(
			ChallengeRound::Action,
			&action,
			playing_bot_name.clone(),
			context,
		) {
			// The bot "challenger" is challenging this action
			let success = self.resolve_challenge(
				action.clone(),
				playing_bot_name.clone(),
				challenger.clone(),
				context,
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

				// THE COUNTER CHALLENGE ROUND
				// On Action::Assassination, Action::ForeignAid and Action::Stealing only
				// Does the target want to counter this action?
				let counter = self.get_bot_by_name(target_name.clone()).on_counter(
					&action,
					playing_bot_name.clone(),
					context,
				);

				Self::log(format_args!(
					"🛑  {} was countered by {}",
					self.get_bot_by_name(playing_bot_name.clone()),
					self.get_bot_by_name(target_name.clone()),
				));

				if counter.is_some() {
					// The bot target_name is countering the action so we now ask the table if anyone would like to challenge this counter
					if let Some(counter_challenge) = self.challenge_round(
						ChallengeRound::Counter,
						&action,
						target_name.clone(),
						context,
					) {
						let counter_card = match action {
							Action::Assassination(_) => Counter::Assassination,
							Action::Stealing(_) => Counter::Stealing,
							Action::Coup(_)
							| Action::ForeignAid
							| Action::Swapping
							| Action::Income
							| Action::Tax => unreachable!(
								"Challenge and counter not called on other actions"
							),
						};
						// The bot counter_challenge.by is challenging this action
						let success = self.resolve_counter(
							counter_card,
							target_name.clone(),
							counter_challenge.clone(),
							context,
						);
						let counter_card_name = match action {
							Action::Assassination(_) => "Assassin",
							Action::Stealing(_) => "Captain or the Ambassador",
							Action::Coup(_)
							| Action::ForeignAid
							| Action::Swapping
							| Action::Income
							| Action::Tax => unreachable!(
								"Challenge and counter not called on other actions"
							),
						};
						if success {
							// The challenge was successful so the player who played the counter get a penalty
							self.penalize_bot(
								target_name,
								&format!(
									"it didn't have the {} to block stealing",
									counter_card_name
								),
								context,
							);
						} else {
							// The challenge was unsuccessful so the player who challenged the counter get a penalty and the action is performed
							self.penalize_bot(
								counter_challenge,
								&format!("{} really did have the {} to block stealing so its challenge was unsuccessful", playing_bot_name, counter_card_name),
								context,
							);
							match action {
								Action::Assassination(_) => self.action_assassination(),
								Action::Stealing(_) => self.action_stealing(),
								Action::Coup(_)
								| Action::ForeignAid
								| Action::Swapping
								| Action::Income
								| Action::Tax => unreachable!(
									"Challenge and counter not called on other actions"
								),
							}
						}
					}
				} else {
					// No counter was played so the action is performed
					match action {
						Action::Assassination(_) => self.action_assassination(),
						Action::Stealing(_) => self.action_stealing(),
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
		} else {
			// No challenge was played so the action is performed
			match action {
				Action::Assassination(_) => self.action_assassination(),
				Action::Stealing(_) => self.action_stealing(),
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

	// All bots (minus the playing bot) are asked if they want to challenge a play
	fn challenge_round(
		&mut self,
		challenge_type: ChallengeRound,
		action: &Action,
		by: String,
		context: &Context,
	) -> Option<String> {
		for bot in self.bots.iter() {
			// skipping the challenger
			if bot.get_name() == by.clone() {
				continue;
			}

			let challenging = match challenge_type {
				ChallengeRound::Action => {
					bot.on_challenge_action_round(action, by.clone(), context)
				},
				ChallengeRound::Counter => {
					bot.on_challenge_counter_round(action, by.clone(), context)
				},
			};

			if challenging {
				self.history.push(match challenge_type {
					ChallengeRound::Action => match action {
						Action::Assassination(_) => History::ChallengeAssassin {
							by: bot.get_name(),
							target: by.clone(),
						},
						Action::Swapping => History::ChallengeAmbassador {
							by: bot.get_name(),
							target: by.clone(),
						},
						Action::Stealing(_) => History::ChallengeCaptain {
							by: bot.get_name(),
							target: by.clone(),
						},
						Action::Tax => History::ChallengeDuke {
							by: bot.get_name(),
							target: by.clone(),
						},
						Action::Coup(_) | Action::ForeignAid | Action::Income => {
							unreachable!("Can't challenge Coup, ForeignAid or Income")
						},
					},
					ChallengeRound::Counter => match action {
						Action::Assassination(_) => History::CounterAssassination {
							by: bot.get_name(),
							target: by.clone(),
						},
						Action::ForeignAid => History::CounterForeignAid {
							by: bot.get_name(),
							target: by.clone(),
						},
						Action::Stealing(_) => History::CounterStealing {
							by: bot.get_name(),
							target: by.clone(),
						},
						Action::Coup(_)
						| Action::Swapping
						| Action::Income
						| Action::Tax => {
							unreachable!("Can't counter Coup, Swapping, Income or Tax")
						},
					},
				});
				Self::log(format_args!(
					"❓  {} was challenged by {}",
					self.get_bot_by_name(by),
					bot
				));
				return Some(bot.get_name());
			}
		}
		None
	}

	// We take a card from a bot and replace it with a new one from the deck
	fn swap_card(&mut self, card: Card, swopee: String) {
		Self::log(format_args!(
			"↬  {} is swapping its card for a new card from the deck",
			self.get_bot_by_name(swopee.clone())
		));
		for bot in self.bots.iter_mut() {
			if bot.get_name() == swopee.clone() {
				bot.set_cards(
					bot.get_cards().into_iter().filter(|c| *c != card).collect(),
				);
				self.discard_pile.push(card);

				let mut new_cards = bot.get_cards().clone();
				new_cards.push(self.deck.pop().unwrap());
				bot.set_cards(new_cards);
			}
		}
	}

	// Someone challenged another bot for playing a card they believe is a bluff
	fn resolve_challenge(
		&mut self,
		action: Action,
		player: String,
		challenger: String,
		context: &Context,
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

		if player.get_cards().contains(&card) {
			Self::log(format_args!(
				"👍  The challenge was successful because {} didn't have the {:?}",
				player, card
			));
			self.card_loss(player.get_name(), context);
			false
		} else {
			Self::log(format_args!(
				"👎  The challenge was unsuccessful because {} did have the {:?}",
				player, card
			));
			self.card_loss(challenger.get_name(), context);
			true
		}
	}

	// A bot is countering another bots action against them
	fn resolve_counter(
		&mut self,
		counter: Counter,
		counterer: String,
		challenger: String,
		context: &Context,
	) -> bool {
		self.history.push(match counter {
			Counter::Assassination => History::CounterAssassination {
				by: counterer.clone(),
				target: challenger.clone(),
			},
			Counter::ForeignAid => History::CounterForeignAid {
				by: counterer.clone(),
				target: challenger.clone(),
			},
			Counter::Stealing => History::CounterStealing {
				by: counterer.clone(),
				target: challenger.clone(),
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
			.join(" and the ");

		if cards.iter().any(|&card| counterer.get_cards().contains(&card)) {
			Self::log(format_args!(
				"👍  The counter was successful because {} didn't have the {:?}",
				counterer, card_string
			));
			self.card_loss(counterer.get_name(), context);
			false
		} else {
			Self::log(format_args!(
				"👎  The counter was unsuccessful because {} did have the {:?}",
				counterer, card_string
			));
			self.card_loss(challenger.get_name(), context);
			true
		}
	}

	/// Play n number of rounds and tally up the score in the CLI
	pub fn _looping(&mut self) {
		todo!();
	}

	// *******************************| Actions |****************************** //
	fn action_assassination(&self) {
		Self::log(format_args!(
			"🃏  {} assassinates",
			self.bots[self.playing_bots[self.turn]],
			// self.get_bot_by_name(target.clone())
		));
		// Self::log(format_args!("🃏  {} plays a card", x));
		// todo!()
	}

	fn action_couping(
		&mut self,
		target: String,
		playing_bot_coins: u8,
		playing_bot_name: String,
		context: &Context,
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
			self.history.push(History::ActionCoup {
				by: playing_bot_name.clone(),
				target: target_bot_name.clone(),
			});
			Self::log(format_args!(
				"🃏  {} coups {}",
				self.bots[self.playing_bots[self.turn]],
				self.get_bot_by_name(target.clone())
			));
			self.card_loss(target_bot_name, context);
		}
	}

	fn _action_foraign_aid(&self) {
		// Self::log(format_args!("🃏  {} plays a card", x));
		todo!()
	}

	fn _action_swapping(&self) {
		// Self::log(format_args!("🃏  {} plays a card", x));
		// console.log(`↬   ${this.getAvatar(challengee)} put the ${style.yellow(card)} back in the deck and drew a new card`);
		todo!()
	}

	fn action_income(&mut self, playing_bot_coins: u8, playing_bot_name: String) {
		// Adding the coin to the bot
		self.bots[self.playing_bots[self.turn]].set_coins(playing_bot_coins + 1);

		// Logging
		self.history.push(History::ActionIncome {
			by: playing_bot_name.clone(),
		});
		Self::log(format_args!(
			"🃏  {} takes \x1b[33ma coin\x1b[39m",
			self.bots[self.playing_bots[self.turn]]
		));
	}

	fn action_stealing(&self) {
		// Self::log(format_args!("🃏  {} plays a card", x));
		todo!()
	}

	fn _action_tax(&self) {
		// Self::log(format_args!("🃏  {} plays a card", x));
		todo!()
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
		let coup = Coup::new(vec![
			Box::new(StaticBot::new(String::from("Player 1")))
				as Box<dyn BotInterface>,
			Box::new(StaticBot::new(String::from("Player 2")))
				as Box<dyn BotInterface>,
		]);

		assert_eq!(coup.bots[0].get_cards(), vec![]);
		assert_eq!(coup.bots[1].get_cards(), vec![]);
		assert_eq!(coup.playing_bots, vec![]);
		assert_eq!(coup.deck, vec![]);
		assert_eq!(coup.discard_pile, vec![]);
		assert_eq!(coup.history, vec![]);
		assert_eq!(
			coup.score,
			vec![(String::from("Player 1"), 0), (String::from("Player 2"), 0)]
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
			Box::new(StaticBot::new(String::from("Player 1")))
				as Box<dyn BotInterface>,
			Box::new(StaticBot::new(String::from("Player 2")))
				as Box<dyn BotInterface>,
			Box::new(StaticBot::new(String::from("Player 3")))
				as Box<dyn BotInterface>,
			Box::new(StaticBot::new(String::from("Player 4")))
				as Box<dyn BotInterface>,
			Box::new(StaticBot::new(String::from("Player 5")))
				as Box<dyn BotInterface>,
			Box::new(StaticBot::new(String::from("Player 6")))
				as Box<dyn BotInterface>,
			Box::new(StaticBot::new(String::from("Player 7")))
				as Box<dyn BotInterface>,
			Box::new(StaticBot::new(String::from("Player 8")))
				as Box<dyn BotInterface>,
		]);
		coup.setup();

		assert_eq!(coup.bots[0].get_cards().len(), 2);
		assert_eq!(coup.bots[0].get_coins(), 2);
		assert_eq!(coup.bots[1].get_cards().len(), 2);
		assert_eq!(coup.bots[1].get_coins(), 2);
		assert_eq!(coup.playing_bots.len(), 6);
		assert_eq!(coup.deck.len(), 3);
		assert_eq!(coup.discard_pile, vec![]);
		assert_eq!(coup.turn, 0);
		assert_eq!(coup.moves, 0);
	}

	#[test]
	fn test_get_bot_by_name() {
		let mut coup = Coup::new(vec![
			Box::new(StaticBot::new(String::from("Player 1")))
				as Box<dyn BotInterface>,
			Box::new(StaticBot::new(String::from("Player 2")))
				as Box<dyn BotInterface>,
		]);
		coup.setup();

		assert_eq!(
			coup.get_bot_by_name(String::from("Player 2")).get_name(),
			String::from("Player 2")
		);
		assert_eq!(
			coup.get_bot_by_name(String::from("Player 1")).get_name(),
			String::from("Player 1")
		);
	}

	#[test]
	fn test_get_other_bots() {
		let mut coup = Coup::new(vec![
			Box::new(StaticBot::new(String::from("Player 1")))
				as Box<dyn BotInterface>,
			Box::new(StaticBot::new(String::from("Player 2")))
				as Box<dyn BotInterface>,
			Box::new(StaticBot::new(String::from("Player 3")))
				as Box<dyn BotInterface>,
			Box::new(StaticBot::new(String::from("Player 4")))
				as Box<dyn BotInterface>,
			Box::new(StaticBot::new(String::from("Player 5")))
				as Box<dyn BotInterface>,
		]);
		coup.setup();

		coup.playing_bots = vec![0, 1, 2, 3, 4];
		assert!(!coup.get_other_bots().contains(&OtherBot {
			name: String::from("Player 1"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("Player 2"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("Player 3"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("Player 4"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("Player 5"),
			coins: 2,
			cards: 2
		}));

		coup.playing_bots = vec![4, 3, 2, 1, 0];
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("Player 1"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("Player 2"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("Player 3"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("Player 4"),
			coins: 2,
			cards: 2
		}));
		assert!(!coup.get_other_bots().contains(&OtherBot {
			name: String::from("Player 5"),
			coins: 2,
			cards: 2
		}));

		coup.turn = 2;
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("Player 1"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("Player 2"),
			coins: 2,
			cards: 2
		}));
		assert!(!coup.get_other_bots().contains(&OtherBot {
			name: String::from("Player 3"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("Player 4"),
			coins: 2,
			cards: 2
		}));
		assert!(coup.get_other_bots().contains(&OtherBot {
			name: String::from("Player 5"),
			coins: 2,
			cards: 2
		}));
	}

	#[test]
	fn test_card_loss() {
		let mut coup = Coup::new(vec![
			Box::new(StaticBot::new(String::from("Player 1")))
				as Box<dyn BotInterface>,
			Box::new(StaticBot::new(String::from("Player 2")))
				as Box<dyn BotInterface>,
		]);
		coup.setup();

		coup.bots[0].set_cards(vec![Card::Ambassador, Card::Duke]);
		coup.bots[1].set_cards(vec![Card::Assassin, Card::Captain]);

		coup.card_loss(
			String::from("Player 2"),
			&Context {
				other_bots: coup.get_other_bots(),
				discard_pile: vec![],
				history: vec![],
				score: vec![],
			},
		);

		assert_eq!(coup.bots[0].get_cards(), vec![Card::Ambassador, Card::Duke]);
		assert_eq!(coup.bots[1].get_cards(), vec![Card::Assassin]);
		assert_eq!(coup.discard_pile, vec![Card::Captain]);
	}

	#[test]
	fn test_card_loss_faulty_bot() {
		struct TestBot {
			pub name: String,
			pub coins: u8,
			pub cards: Vec<Card>,
		}
		impl TestBot {
			pub fn new(name: String) -> Self {
				Self {
					name,
					coins: 2,
					cards: vec![],
				}
			}
		}
		impl BotInterface for TestBot {
			fn get_name(&self) -> String {
				self.name.clone()
			}
			fn get_coins(&self) -> u8 {
				self.coins
			}
			fn set_coins(&mut self, coins: u8) {
				self.coins = coins;
			}
			fn get_cards(&self) -> Vec<Card> {
				(*self.cards).to_vec()
			}
			fn set_cards(&mut self, cards: Vec<Card>) {
				self.cards = cards;
			}
			fn on_card_loss(&self, _context: &Context) -> Card {
				Card::Duke
			}
		}

		let mut coup = Coup::new(vec![
			Box::new(StaticBot::new(String::from("Player 1")))
				as Box<dyn BotInterface>,
			Box::new(TestBot::new(String::from("Player 2"))) as Box<dyn BotInterface>,
		]);
		coup.setup();

		coup.bots[0].set_cards(vec![Card::Ambassador, Card::Duke]);
		coup.bots[1].set_cards(vec![Card::Assassin, Card::Captain]);

		coup.card_loss(
			String::from("Player 2"),
			&Context {
				other_bots: coup.get_other_bots(),
				discard_pile: vec![],
				history: vec![],
				score: vec![],
			},
		);

		assert_eq!(coup.bots[0].get_cards(), vec![Card::Ambassador, Card::Duke]);
		assert_eq!(coup.bots[1].get_cards(), vec![]);
		assert_eq!(coup.discard_pile, vec![Card::Captain, Card::Assassin]);
	}

	#[test]
	fn test_swap_card() {
		let mut coup = Coup::new(vec![
			Box::new(StaticBot::new(String::from("Player 1")))
				as Box<dyn BotInterface>,
			Box::new(StaticBot::new(String::from("Player 2")))
				as Box<dyn BotInterface>,
		]);
		coup.setup();

		coup.bots[0].set_cards(vec![Card::Ambassador, Card::Duke]);
		coup.bots[1].set_cards(vec![Card::Assassin, Card::Captain]);
		coup.deck = vec![Card::Ambassador, Card::Captain];

		assert_eq!(coup.bots[0].get_cards(), vec![Card::Ambassador, Card::Duke]);
		assert_eq!(coup.bots[1].get_cards(), vec![Card::Assassin, Card::Captain]);
		assert_eq!(coup.deck, vec![Card::Ambassador, Card::Captain]);
		assert_eq!(coup.discard_pile, vec![]);

		coup.swap_card(Card::Ambassador, String::from("Player 1"));

		assert_eq!(coup.bots[0].get_cards(), vec![Card::Duke, Card::Captain]);
		assert_eq!(coup.bots[1].get_cards(), vec![Card::Assassin, Card::Captain]);
		assert_eq!(coup.deck, vec![Card::Ambassador]);
		assert_eq!(coup.discard_pile, vec![Card::Ambassador]);
	}
}
