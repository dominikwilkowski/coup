use coup_cli::{
	bot::BotInterface,
	bots::{HonestBot, StaticBot},
	Card, Coup,
};

fn main() {
	let mut coup_game = Coup::new(vec![
		Box::new(StaticBot::new(String::from("Bot 1"))),
		Box::new(HonestBot::new(String::from("Bot 2"))),
		Box::new(StaticBot::new(String::from("Bot 3"))),
	]);

	coup_game.play();
	println!("{:#?}", coup_game);

	let mut bot =
		Box::new(StaticBot::new(String::from("Bot 3"))) as Box<dyn BotInterface>;
	bot.set_cards(vec![Card::Duke]);
	println!("{}", bot);
}
