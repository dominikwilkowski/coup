use coup::{
	bots::{HonestBot, StaticBot},
	Coup,
};

fn main() {
	let mut coup_game = Coup::new(vec![
		Box::new(StaticBot::new(String::from("Charles"))),
		Box::new(HonestBot::new(String::from("Tici"))),
		Box::new(StaticBot::new(String::from("Novini"))),
		Box::new(HonestBot::new(String::from("Dom"))),
	]);

	coup_game.play();
	// println!("{:#?}", coup_game);
}
