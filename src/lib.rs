use rand::{seq::SliceRandom, thread_rng};

pub enum Card {
	Duke,
	Assassin,
	Ambassador,
	Captain,
	Contessa,
}

pub struct Bot {
	pub name: String,
	pub coins: u8,
	cards: Vec<Card>,
}

pub enum History {
	ActionSwapping { initiator: Bot },
	ActionStealing { initiator: Bot, target: Bot },
	ActionForeignAid { initiator: Bot },
	ActionTax { initiator: Bot },
	ActionAssassination { initiator: Bot, target: Bot },
	ActionIncome { initiator: Bot },
	ActionCoup { initiator: Bot, target: Bot },
	ChallengeSwapping { initiator: Bot, target: Bot },
	ChallengeStealing { initiator: Bot, target: Bot },
	ChallengeTax { initiator: Bot, target: Bot },
	ChallengeAssassination { initiator: Bot, target: Bot },
	ChallengeBlockingForeignAid { initiator: Bot, target: Bot },
	ChallengeBlockingAssassination { initiator: Bot, target: Bot },
	ChallengeBlockingStealing { initiator: Bot, target: Bot },
	CounterActionBlockingForeignAid { initiator: Bot, target: Bot },
	CounterActionBlockingAssassination { initiator: Bot, target: Bot },
	CounterActionBlockingStealing { initiator: Bot, target: Bot },
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
	}

	pub fn _play(mut self) {}
	pub fn _looping(mut self) {}
}
