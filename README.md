# Coup

<p align="center"><img width="764" src="assets/coup.png"></p>

<p align="center">This is a CLI implementation of the game of <a href="http://gamegrumps.wikia.com/wiki/Coup">COUP</a>.</p>

<p align="center">
	<a href="https://crates.io/crates/coup"><img src="https://img.shields.io/crates/v/coup.svg" alt="crates badge"></a>
	<a href="https://docs.rs/coup"><img src="https://docs.rs/coup/badge.svg" alt="crates docs tests"></a>
	<a href="https://github.com/dominikwilkowski/coup/actions/workflows/testing.yml"><img src="https://github.com/dominikwilkowski/coup/actions/workflows/testing.yml/badge.svg" alt="build status"></a>
</p>

This app is designed as a code challenge.
It challenges you to write a bot that plays [COUP](http://gamegrumps.wikia.com/wiki/Coup) against other bots.

The idea is to have three rounds of (1,000,000) games to find the winner (sum all scores).
Between each round you have time to make adjustments to your bot.

## How does this work?

- [Rules](#rules)
- [Scoring](#scoring)
- [How to run the game](#how-to-run-the-game)
- [How do I build a bot](#how-do-i-build-a-bot)
- [How does the engine work](#how-does-the-engine-work)

## Rules

1. No changes to engine
1. Name of bots don't change between rounds (so you can target specific bots)
1. No data sharing between games within a round
1. No file access to other bots
1. No changing other bots
1. No internet access or calls to OpenAI
1. Do not output to `stdout` or `stderr`

## Scoring

Each game is a zero-sum-game in terms of score.
That means the amount of negative points given to losers + the amount of
positive points given to winners equals to zero.

The score is determined by the number of players (can't be more than 6 per game)
and winners (there are instances where the game can stall in a stale-mate which
the engine will stop and nominate multiple winners for).
Each game will take a max of 6 bots that are randomly elected.
Those who win get a positive score, those who lose will get a negative score.

- Score for losers: `-1/(players-1)`
- Score for winners: `∑losers/winners`

## How to run the game

You can run the game in two modes: `play` and `loop`.

### Play mode

<p align="center">
	<img width="882" src="assets/play.png">
</p>

The `play` mode will play a single game and nominate (a) winner(s) at the end.

```rust
use coup::{
	bots::{HonestBot, RandomBot, StaticBot},
	Coup,
};

fn main() {
	let mut coup_game = Coup::new(vec![
		Box::new(StaticBot),
		Box::new(StaticBot),
		Box::new(HonestBot),
		Box::new(HonestBot),
		Box::new(RandomBot),
		Box::new(RandomBot),
	]);

	coup_game.play();
}
```

### Loop mode

<p align="center">
	<img width="680" src="assets/loop.gif">
</p>

The `loop` mode will play `n` amount of games and sum all score and nominate (a)
winner(s) at the end

```rust
use coup::{
	bots::{HonestBot, RandomBot, StaticBot},
	Coup,
};

fn main() {
	let mut coup_game = Coup::new(vec![
		Box::new(StaticBot),
		Box::new(StaticBot),
		Box::new(HonestBot),
		Box::new(HonestBot),
		Box::new(RandomBot),
		Box::new(RandomBot),
	]);

	coup_game.looping(1_000_000);
}
```

## How do I build a bot

Implement the `BotInterface` and override the default implementations of each of
the methods you'd like to take control over.
The default implementation are the methods of the `StaticBot` which only takes
`Income` and is forced to coup by the engine if it accumulated more or equal to
10 coins. It does not challenge, counter or counter challenge.

The simplest way to build a bot by falling back to static behavior for each method would be:

```rust
use coup::bot::BotInterface;

pub struct MyBot;

impl BotInterface for MyBot {
	fn get_name(&self) -> String {
		String::from("MyBot")
	}
}
```

_(This is what the `StaticBot` is.)_

From there you can choose which, if not all, of the below methods you change to
make this bot your own.

### Methods of the bot

The methods of `BotInterface` that will define the behavior of your bot.

- `get_name` – Called only once at the instantiation of the Coup game to identify your bot
- `on_turn` – Called when it's your turn to decide what to do
- `on_auto_coup` – Called when you have equal to or more than 10 coins and must coup.
- `on_challenge_action_round` – Called when another bot played an action and everyone gets to decide whether they want to challenge that action.
- `on_counter` – Called when someone played something that can be countered with a card you may have.
- `on_challenge_counter_round` – Called when a bot played a counter. Now everyone gets to decided whether they want to challenge that counter card.
- `on_swapping_cards` – Called when you played your ambassador and now need to decide which cards you want to keep.
- `on_card_loss` – Called when you lost a card and now must decide which one you want to lose

### The context

Each function gets `context` passed in which will contain below infos:

| key            | description                                                                                                                                                                                     |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `name`         | Your bots name after it was de-duped by the engine. This means if you have multiple bots with the same name they get a space and a number appended to their name which is used as an identifier |
| `cards`        | Your cards/influences you still have                                                                                                                                                            |
| `coins`        | Your coins                                                                                                                                                                                      |
| `playing_bots` | A list of all playing bots this round                                                                                                                                                           |
| `discard_pile` | A list of all discarded cards so far in the game                                                                                                                                                |
| `history`      | A list of each event that has happened in this game so far                                                                                                                                      |
| `score`        | The current score of the game                                                                                                                                                                   |

## How does the engine work

```
match action
	Assassination | Stealing
		=>
			- challenge round
			- counter from target
			- counter challenge
			- action
	Coup | Income
		=>
			- action
	ForeignAid
		=>
			- counter round from everyone
			- counter challenge round
			- action
	Swapping | Tax
		=>
			- challenge round
			- action
```
