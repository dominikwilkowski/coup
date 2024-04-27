use coup::{
	bots::{HonestBot, StaticBot},
	Coup,
};

fn main() {
	let mut coup_game = Coup::new(vec![
		Box::new(StaticBot),
		Box::new(HonestBot),
		Box::new(StaticBot),
		Box::new(HonestBot),
	]);

	coup_game.play();
	// println!("{:#?}", coup_game);
}
