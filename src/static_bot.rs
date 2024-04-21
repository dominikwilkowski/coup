use crate::{
	bot::{Bot, BotInterface},
	Action, Card, History,
};

impl BotInterface for Bot {
	fn get_name(&self) -> String {
		self.name.clone()
	}

	fn get_coins(&self) -> u8 {
		self.coins
	}

	fn get_cards(&self) -> Vec<Card> {
		(*self.cards).to_vec()
	}

	fn on_turn(
		&self,
		other_bots: Vec<Bot>,
		discard_pile: Vec<Card>,
		history: History,
	) -> Action {
		Action::Income {
			initiator: self.get_name().clone(),
		}
	}
}
