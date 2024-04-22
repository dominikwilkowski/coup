use coup_cli::{
	bots::{HonestBot, StaticBot},
	Coup,
};

fn main() {
	let mut coup_game = Coup::new(vec![
		Box::new(StaticBot::new(String::from("Bot 1"))),
		Box::new(HonestBot::new(String::from("Bot 2"))),
		Box::new(StaticBot::new(String::from("Bot 3"))),
	]);

	coup_game.start_round();
	println!("{:#?}", coup_game);
}
