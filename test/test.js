const { style } = require('../src/helper.js');
const { COUP } = require('../src/index.js');

// defaults
const makeBots = () => ({
	bot1: {
		onTurn: () => ({}),
		onChallengeActionRound: () => false,
		onCounterAction: () => false,
		onCounterActionRound: () => false,
		onSwappingCards: () => {},
		onCardLoss: ({ myCards }) => myCards[0],
	},
	bot2: {
		onTurn: () => ({}),
		onChallengeActionRound: () => false,
		onCounterAction: () => false,
		onCounterActionRound: () => false,
		onSwappingCards: () => {},
		onCardLoss: ({ myCards }) => myCards[0],
	},
	bot3: {
		onTurn: () => ({}),
		onChallengeActionRound: () => false,
		onCounterAction: () => false,
		onCounterActionRound: () => false,
		onSwappingCards: () => {},
		onCardLoss: ({ myCards }) => myCards[0],
	},
});

const makePlayer = () => ({
	bot1: {
		card1: undefined,
		card2: undefined,
		coins: 0,
	},
	bot2: {
		card1: undefined,
		card2: undefined,
		coins: 0,
	},
	bot3: {
		card1: undefined,
		card2: undefined,
		coins: 0,
	},
});

console.log = () => {};
let pass = true;

const TEST = {
	//   _____     _     _  _   ___   _  _    ___         _
	// |_   _|   /_\   | |/ / |_ _| | \| |  / __|  ___  / |
	//   | |    / _ \  | ' <   | |  | .` | | (_ | |___| | |
	//   |_|   /_/ \_\ |_|\_\ |___| |_|\_|  \___|       |_|
	// bot1 will take one coin
	'taking-1': async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'duke';
		player.bot2.card1 = 'duke';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'taking-1', against: 'bot1' });

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = [];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (game.PLAYER.bot1.coins === 1 && game.PLAYER.bot1.card1 === 'duke' && game.PLAYER.bot2.card1 === 'duke') {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('taking-1')} action`);
	},
	//    __    ___    _   _   ___   ___   _  _    ___
	//  / __|  / _ \  | | | | | _ \ |_ _| | \| |  / __|
	// | (__  | (_) | | |_| | |  _/  | |  | .` | | (_ |
	//  \___|  \___/   \___/  |_|   |___| |_|\_|  \___|
	// bot1 will coup bot2
	couping: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'duke';
		player.bot1.coins = 8;
		player.bot2.card1 = 'duke';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'couping', against: 'bot2' });

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = [];
		game.TURN = 2;

		await game.turn();

		let status = style.red('FAIL');
		if (game.PLAYER.bot1.card1 === 'duke' && game.PLAYER.bot1.coins === 1 && game.PLAYER.bot2.card1 === undefined) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('couping')} action`);
	},
	//  _____     _     _  _   ___   _  _    ___         ___
	// |_   _|   /_\   | |/ / |_ _| | \| |  / __|  ___  |__ /
	//   | |    / _ \  | ' <   | |  | .` | | (_ | |___|  |_ \
	//   |_|   /_/ \_\ |_|\_\ |___| |_|\_|  \___|       |___/
	// bot1 will take three coins with duke
	'taking-31': async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.coins = 0;
		player.bot1.card1 = 'duke';
		player.bot2.card1 = 'duke';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'taking-3', against: 'bot2' });

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = [];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (game.PLAYER.bot1.coins === 3 && game.PLAYER.bot1.card1 === 'duke' && game.PLAYER.bot2.card1 === 'duke') {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('taking-3')} without challenge`);
	},
	// bot1 will take three coins with duke, bot2 calls bot1, bot1 did not have the duke
	'taking-32': async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.coins = 0;
		player.bot1.card1 = 'captain';
		player.bot2.card1 = 'duke';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'taking-3', against: 'bot2' });
		bots.bot2.onChallengeActionRound = () => true;

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = [];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (game.PLAYER.bot1.coins === 0 && game.PLAYER.bot1.card1 === undefined && game.PLAYER.bot2.card1 === 'duke') {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('taking-3')} with successful challenge`);
	},
	// bot1 will take three coins with duke, bot2 calls bot1, bot1 did have the duke
	'taking-33': async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.coins = 0;
		player.bot1.card1 = 'duke';
		player.bot2.card1 = 'duke';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'taking-3', against: 'bot2' });
		bots.bot2.onChallengeActionRound = () => true;

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = [];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (game.PLAYER.bot1.coins === 3 && game.PLAYER.bot1.card1 === 'duke' && game.PLAYER.bot2.card1 === undefined) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('taking-3')} with unsuccessful challenge`);
	},
	//    _     ___   ___     _     ___   ___   ___   _  _     _     _____   ___    ___    _  _
	//   /_\   / __| / __|   /_\   / __| / __| |_ _| | \| |   /_\   |_   _| |_ _|  / _ \  | \| |
	//  / _ \  \__ \ \__ \  / _ \  \__ \ \__ \  | |  | .` |  / _ \    | |    | |  | (_) | | .` |
	// /_/ \_\ |___/ |___/ /_/ \_\ |___/ |___/ |___| |_|\_| /_/ \_\   |_|   |___|  \___/  |_|\_|
	// bot1 will assassinate bot2 with assassin
	assassination1: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'duke';
		player.bot1.coins = 4;
		player.bot2.card1 = 'duke';
		player.bot2.card2 = 'captain';
		player.bot2.coins = 5;

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'assassination', against: 'bot2' });
		bots.bot2.onCardLoss = () => 'captain';

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = [];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === 'duke' &&
			game.PLAYER.bot1.coins === 1 &&
			game.PLAYER.bot2.card1 === 'duke' &&
			game.PLAYER.bot2.card2 === undefined &&
			game.PLAYER.bot2.coins === 5
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('assassination')} without challenge or counter action`);
	},
	// bot1 will assassinate bot2 with assassin, bot2 calls bot1, bot1 did not have the assassin
	assassination2: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'duke';
		player.bot1.coins = 4;
		player.bot2.card1 = 'duke';
		player.bot2.card2 = 'captain';
		player.bot2.coins = 5;

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'assassination', against: 'bot2' });
		bots.bot2.onChallengeActionRound = () => true;

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = [];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === undefined &&
			game.PLAYER.bot1.coins === 1 &&
			game.PLAYER.bot2.card1 === 'duke' &&
			game.PLAYER.bot2.card2 === 'captain' &&
			game.PLAYER.bot2.coins === 5
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('assassination')} with successful challenge`);
	},
	// bot1 will assassinate bot2 with assassin, bot2 calls bot1, bot1 did have the assassin
	assassination3: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'assassin';
		player.bot1.coins = 4;
		player.bot2.card1 = 'duke';
		player.bot2.card2 = 'captain';
		player.bot2.coins = 5;

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'assassination', against: 'bot2' });
		bots.bot2.onChallengeActionRound = () => true;
		bots.bot2.onCardLoss = () => 'captain';

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['duke', 'captain'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 !== undefined &&
			game.PLAYER.bot1.coins === 1 &&
			game.PLAYER.bot2.card1 === undefined &&
			game.PLAYER.bot2.card2 === undefined &&
			game.PLAYER.bot2.coins === 5
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('assassination')} with unsuccessful challenge`);
	},
	// bot1 will assassinate bot2 with assassin, bot2 says it has the contessa, bot1 is fine with that
	assassination4: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'assassin';
		player.bot1.coins = 4;
		player.bot2.card1 = 'duke';
		player.bot2.card2 = 'captain';
		player.bot2.coins = 5;

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'assassination', against: 'bot2' });
		bots.bot2.onCounterAction = () => 'contessa';

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['duke', 'captain'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === 'assassin' &&
			game.PLAYER.bot1.coins === 1 &&
			game.PLAYER.bot2.card1 === 'duke' &&
			game.PLAYER.bot2.card2 === 'captain' &&
			game.PLAYER.bot2.coins === 5
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('assassination')} with counter action but no counter challenge`);
	},
	// bot1 will assassinate bot2 with assassin, bot2 says it has the contessa, bot1 is challenging bot2, bot2 did not have the contessa
	assassination5: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'assassin';
		player.bot1.coins = 4;
		player.bot2.card1 = 'duke';
		player.bot2.card2 = 'captain';
		player.bot2.coins = 5;

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'assassination', against: 'bot2' });
		bots.bot2.onCounterAction = () => 'contessa';
		bots.bot1.onCounterActionRound = () => true;

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['duke', 'captain'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === 'assassin' &&
			game.PLAYER.bot1.coins === 1 &&
			game.PLAYER.bot2.card1 === undefined &&
			game.PLAYER.bot2.card2 === undefined &&
			game.PLAYER.bot2.coins === 5
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('assassination')} with counter action and successful counter challenge`);
	},
	// bot1 will assassinate bot2 with assassin, bot2 says it has the contessa, bot1 is challenging bot2, bot2 did have the contessa
	assassination6: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'assassin';
		player.bot1.coins = 4;
		player.bot2.card1 = 'duke';
		player.bot2.card2 = 'contessa';
		player.bot2.coins = 5;

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'assassination', against: 'bot2' });
		bots.bot2.onCounterAction = () => 'contessa';
		bots.bot1.onCounterActionRound = () => true;

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['duke', 'captain'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === undefined &&
			game.PLAYER.bot1.coins === 1 &&
			game.PLAYER.bot2.card1 === 'duke' &&
			game.PLAYER.bot2.card2 !== undefined &&
			game.PLAYER.bot2.coins === 5
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('assassination')} with counter action and unsuccessful counter challenge`);
	},
	//  ___   _____   ___     _     _      ___   _  _    ___
	// / __| |_   _| | __|   /_\   | |    |_ _| | \| |  / __|
	// \__ \   | |   | _|   / _ \  | |__   | |  | .` | | (_ |
	// |___/   |_|   |___| /_/ \_\ |____| |___| |_|\_|  \___|
	// bot1 will steal from bot2 with captain
	stealing1: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.coins = 1;
		player.bot1.card1 = 'duke';
		player.bot2.coins = 5;
		player.bot2.card1 = 'duke';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'stealing', against: 'bot2' });

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = [];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.coins === 3 &&
			game.PLAYER.bot1.card1 === 'duke' &&
			game.PLAYER.bot2.coins === 3 &&
			game.PLAYER.bot2.card1 === 'duke'
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('stealing')} without challenge or counter action`);
	},
	// bot1 will steal from bot2 with captain, bot2 calls bot1, bot1 did not have the captain
	stealing2: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.coins = 1;
		player.bot1.card1 = 'duke';
		player.bot2.coins = 5;
		player.bot2.card1 = 'duke';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'stealing', against: 'bot2' });
		bots.bot2.onChallengeActionRound = () => true;

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = [];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.coins === 1 &&
			game.PLAYER.bot1.card1 === undefined &&
			game.PLAYER.bot2.coins === 5 &&
			game.PLAYER.bot2.card1 === 'duke'
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('stealing')} with successful challenge`);
	},
	// bot1 will steal from bot2 with captain, bot2 calls bot1, bot1 did have the captain
	stealing3: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.coins = 1;
		player.bot1.card1 = 'captain';
		player.bot2.coins = 5;
		player.bot2.card1 = 'duke';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'stealing', against: 'bot2' });
		bots.bot2.onChallengeActionRound = () => true;

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['duke', 'assassin'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.coins === 3 &&
			game.PLAYER.bot1.card1 !== undefined &&
			game.PLAYER.bot2.coins === 3 &&
			game.PLAYER.bot2.card1 === undefined
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('stealing')} with unsuccessful challenge`);
	},
	// bot1 will steal from bot2 with captain, bot2 says it has the captain|ambassador, bot1 is fine with that
	stealing4: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.coins = 1;
		player.bot1.card1 = 'captain';
		player.bot2.coins = 5;
		player.bot2.card1 = 'duke';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'stealing', against: 'bot2' });
		bots.bot2.onCounterAction = () => 'captain';

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['duke', 'assassin'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.coins === 1 &&
			game.PLAYER.bot1.card1 === 'captain' &&
			game.PLAYER.bot2.coins === 5 &&
			game.PLAYER.bot2.card1 === 'duke'
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('stealing')} with counter action but no counter challenge`);
	},
	// bot1 will steal from bot2 with captain, bot2 says it has the captain|ambassador, bot1 is challenging bot2, bot2 did not have the captain|ambassador
	stealing5: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.coins = 1;
		player.bot1.card1 = 'captain';
		player.bot2.coins = 5;
		player.bot2.card1 = 'duke';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'stealing', against: 'bot2' });
		bots.bot2.onCounterAction = () => 'captain';
		bots.bot1.onCounterActionRound = () => true;

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['duke', 'assassin'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.coins === 3 &&
			game.PLAYER.bot1.card1 === 'captain' &&
			game.PLAYER.bot2.coins === 3 &&
			game.PLAYER.bot2.card1 === undefined
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('stealing')} with counter action and successful counter challenge`);
	},
	// bot1 will steal from bot2 with captain, bot2 says it has the captain|ambassador, bot1 is challenging bot2, bot2 did have the captain|ambassador
	stealing6: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.coins = 1;
		player.bot1.card1 = 'captain';
		player.bot2.coins = 5;
		player.bot2.card1 = 'captain';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'stealing', against: 'bot2' });
		bots.bot2.onCounterAction = () => 'captain';
		bots.bot1.onCounterActionRound = () => true;

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['duke', 'assassin'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.coins === 1 &&
			game.PLAYER.bot1.card1 === undefined &&
			game.PLAYER.bot2.coins === 5 &&
			game.PLAYER.bot2.card1 !== undefined
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('stealing')} with counter action and unsuccessful counter challenge`);
	},
	//  ___  __      __    _     ___   ___   ___   _  _    ___
	// / __| \ \    / /   /_\   | _ \ | _ \ |_ _| | \| |  / __|
	// \__ \  \ \/\/ /   / _ \  |  _/ |  _/  | |  | .` | | (_ |
	// |___/   \_/\_/   /_/ \_\ |_|   |_|   |___| |_|\_|  \___|
	// bot1 will swap cards with ambassador
	swapping1: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'assassin';
		player.bot1.card2 = 'contessa';
		player.bot2.card1 = 'ambassador';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'swapping', against: 'bot2' });
		bots.bot1.onSwappingCards = ({ newCards }) => newCards;

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['duke', 'captain'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === 'captain' &&
			game.PLAYER.bot1.card2 === 'duke' &&
			game.PLAYER.bot2.card1 === 'ambassador'
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('swapping')} without challenge`);
	},
	// bot1 will swap cards with ambassador, bot2 calls bot1, bot1 did not have the ambassador
	swapping2: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'assassin';
		player.bot1.card2 = 'contessa';
		player.bot2.card1 = 'ambassador';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'swapping', against: 'bot2' });
		bots.bot1.onSwappingCards = ({ newCards }) => newCards;
		bots.bot2.onChallengeActionRound = () => true;

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['duke', 'captain'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === undefined &&
			game.PLAYER.bot1.card2 === 'contessa' &&
			game.PLAYER.bot2.card1 === 'ambassador'
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('swapping')} with successful challenge`);
	},
	// bot1 will swap cards with ambassador, bot2 calls bot1, bot1 did have the ambassador
	swapping3: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'ambassador';
		player.bot1.card2 = 'contessa';
		player.bot2.card1 = 'ambassador';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'swapping', against: 'bot2' });
		bots.bot1.onSwappingCards = ({ newCards }) => newCards;
		bots.bot2.onChallengeActionRound = () => true;

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['duke', 'captain'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 !== undefined &&
			game.PLAYER.bot1.card2 !== undefined &&
			game.PLAYER.bot2.card1 === undefined
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  ${style.yellow('swapping')} with unsuccessful challenge`);
	},
	//  ___    ___    ___   ___   ___    ___   _  _           _     ___   ___
	// | __|  / _ \  | _ \ | __| |_ _|  / __| | \| |  ___    /_\   |_ _| |   \
	// | _|  | (_) | |   / | _|   | |  | (_ | | .` | |___|  / _ \   | |  | |) |
	// |_|    \___/  |_|_\ |___| |___|  \___| |_|\_|       /_/ \_\ |___| |___/
	// bot1 will take bot2 foreign aid
	'foreign-aid1': async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'duke';
		player.bot2.card1 = 'duke';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'foreign-aid', against: 'bo2' });

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = [];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (game.PLAYER.bot1.coins === 2 && game.PLAYER.bot1.card1 === 'duke' && game.PLAYER.bot2.card1 === 'duke') {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${style.red(status)}  ${style.yellow('foreign-aid')} without counter action`);
	},
	// bot1 will take bot2 foreign aid, bot2 says it has the duke, bot1 is fine with that
	'foreign-aid2': async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.coins = 0;
		player.bot1.card1 = 'duke';
		player.bot2.card1 = 'duke';
		player.bot3.card1 = 'captain';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'foreign-aid', against: 'bo2' });
		bots.bot3.onCounterAction = () => 'duke';

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = [];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.coins === 0 &&
			game.PLAYER.bot1.card1 === 'duke' &&
			game.PLAYER.bot2.card1 === 'duke' &&
			game.PLAYER.bot3.card1 === 'captain'
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${style.red(status)}  ${style.yellow('foreign-aid')} with counter action and no counter challenge`);
	},
	// bot1 will take bot2 foreign aid, bot2 says it has the duke, bot1 calls bot2, bot2 did not have the duke
	'foreign-aid3': async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.coins = 0;
		player.bot1.card1 = 'duke';
		player.bot2.card1 = 'duke';
		player.bot3.card1 = 'captain';

		const bots = makeBots();
		bots.bot1.onTurn = () => ({ action: 'foreign-aid', against: 'bo2' });
		bots.bot3.onCounterAction = () => 'duke';
		bots.bot1.onCounterActionRound = () => true;

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = [];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.coins === 2 &&
			game.PLAYER.bot1.card1 === 'duke' &&
			game.PLAYER.bot2.card1 === 'duke' &&
			game.PLAYER.bot3.card1 === undefined
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(
			`${style.red(status)}  ${style.yellow('foreign-aid')} with counter action and successful counter challenge`
		);
	},
	// bot1 will take bot2 foreign aid, bot2 says it has the duke, bot1 calls bot2, bot2 did have the duke
	'foreign-aid4': async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.coins = 0;
		player.bot1.card1 = 'duke';
		player.bot2.card1 = 'duke';
		player.bot3.card1 = 'duke';

		const bots = makeBots();
		let runs = [];
		bots.bot1.onTurn = () => ({ action: 'foreign-aid', against: 'bo2' });
		bots.bot1.onCounterAction = () => {
			runs.push('bot1');
			return 'duke';
		};
		bots.bot2.onCounterAction = () => {
			runs.push('bot2');
			return 'duke';
		};
		bots.bot3.onCounterAction = () => {
			runs.push('bot3');
			return 'duke';
		};
		bots.bot1.onCounterActionRound = () => true;

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = [];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.coins === 0 &&
			game.PLAYER.bot1.card1 === undefined &&
			game.PLAYER.bot2.card1 === 'duke' &&
			game.PLAYER.bot3.card1 !== undefined &&
			runs.length === 1 &&
			!runs.includes('bot1') &&
			!runs.includes('bot3')
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(
			`${style.red(status)}  ${style.yellow('foreign-aid')} with counter action and unsuccessful counter challenge`
		);
	},

	'challenge-only-once': () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'assassin';
		player.bot1.card2 = 'captain';
		player.bot1.coins = 1;
		player.bot2.card1 = 'captain';
		player.bot2.coins = 3;
		player.bot3.card1 = 'duke';

		const bots = makeBots();
		let runs = 0;
		bots.bot2.onCounterAction = () => 'captain';
		bots.bot1.onCardLoss = () => 'assassin';
		bots.bot1.onCounterActionRound = () => {
			runs++;
			return true;
		};
		bots.bot2.onCounterActionRound = () => {
			runs++;
			return true;
		};
		bots.bot3.onCounterActionRound = () => {
			runs++;
			return true;
		};

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['contessa'];
		game.TURN = 0;

		game.runChallenges({ action: 'stealing', player: 'bot1', target: 'bot2' });

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === undefined &&
			game.PLAYER.bot1.card2 === 'captain' &&
			game.PLAYER.bot1.coins === 1 &&
			game.PLAYER.bot2.card1 !== undefined &&
			game.PLAYER.bot2.coins === 3 &&
			game.PLAYER.bot3.card1 === 'duke' &&
			game.DECK.length === 1 &&
			runs === 1
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  an unsuccessful counter action round yields punishment`);
	},
	swapCards1: () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'duke';
		player.bot1.card2 = 'contessa';

		game.PLAYER = player;
		game.DECK = ['duke'];

		game.swapCards({
			chosenCards: ['assassin', 'captain'],
			player: 'bot1',
			newCards: ['captain', 'assassin'],
		});

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === 'assassin' &&
			game.PLAYER.bot1.card2 === 'captain' &&
			game.DECK.includes('duke') &&
			game.DECK.includes('duke') &&
			game.DECK.includes('contessa')
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  swapCards merges cards correctly`);
	},
	swapCards2: () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = undefined;
		player.bot1.card2 = 'contessa';

		game.PLAYER = player;
		game.DECK = ['ambassador'];

		game.swapCards({
			chosenCards: ['ambassador', 'captain'],
			player: 'bot1',
			newCards: ['captain', 'duke'],
		});

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === 'captain' &&
			game.PLAYER.bot1.card2 === undefined &&
			game.DECK.includes('ambassador') &&
			game.DECK.includes('contessa') &&
			game.DECK.includes('duke')
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  swapCards merges cards correctly even with one card`);
	},
	swapCards3: () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'contessa';
		player.bot1.card2 = undefined;

		game.PLAYER = player;
		game.DECK = [];

		game.swapCards({
			chosenCards: ['ambassador', 'ambassador'],
			player: 'bot1',
			newCards: ['captain', 'duke'],
		});

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === undefined &&
			game.PLAYER.bot1.card2 === undefined &&
			game.DECK.includes('contessa') &&
			game.DECK.includes('captain') &&
			game.DECK.includes('duke')
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  swapCards merges cards correctly even when given cards are invalid`);
	},
	checkParameters1: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'assassin';
		player.bot1.card2 = 'captain';
		player.bot1.coins = 1;
		player.bot2.card1 = 'captain';
		player.bot2.coins = 3;
		player.bot3.card1 = 'duke';

		const bots = makeBots();
		let output;
		bots.bot1.onTurn = (param) => {
			output = param;
			return { action: 'foreign-aid' };
		};

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['contessa'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === 'assassin' &&
			game.PLAYER.bot1.card2 === 'captain' &&
			game.PLAYER.bot1.coins === 3 &&
			game.PLAYER.bot2.card1 === 'captain' &&
			game.PLAYER.bot2.coins === 3 &&
			game.PLAYER.bot3.card1 === 'duke' &&
			game.DECK.length === 1 &&
			output.history.length === 0 &&
			output.myCards[0] === 'assassin' &&
			output.myCards[1] === 'captain' &&
			output.myCoins === 1
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  we get the right parameters passed in for onTurn`);
	},
	checkParameters2: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'assassin';
		player.bot1.card2 = 'captain';
		player.bot1.coins = 1;
		player.bot2.card1 = 'captain';
		player.bot2.coins = 3;
		player.bot3.card1 = 'duke';

		const bots = makeBots();
		let output;
		bots.bot1.onTurn = () => ({ action: 'stealing', against: 'bot2' });
		bots.bot2.onChallengeActionRound = (param) => {
			output = param;
			return false;
		};

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['contessa'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === 'assassin' &&
			game.PLAYER.bot1.card2 === 'captain' &&
			game.PLAYER.bot1.coins === 3 &&
			game.PLAYER.bot2.card1 === 'captain' &&
			game.PLAYER.bot2.coins === 1 &&
			game.PLAYER.bot3.card1 === 'duke' &&
			game.DECK.length === 1 &&
			output.myCards[0] === 'captain' &&
			output.myCoins === 3
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  we get the right parameters passed in for onChallengeActionRound`);
	},
	checkParameters3: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'assassin';
		player.bot1.card2 = 'captain';
		player.bot1.coins = 1;
		player.bot2.card1 = 'captain';
		player.bot2.coins = 3;
		player.bot3.card1 = 'duke';

		const bots = makeBots();
		let output;
		bots.bot1.onTurn = () => ({ action: 'foreign-aid' });
		bots.bot2.onCounterAction = (param) => {
			output = param;
			return false;
		};

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['contessa'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === 'assassin' &&
			game.PLAYER.bot1.card2 === 'captain' &&
			game.PLAYER.bot1.coins === 3 &&
			game.PLAYER.bot2.card1 === 'captain' &&
			game.PLAYER.bot2.coins === 3 &&
			game.PLAYER.bot3.card1 === 'duke' &&
			game.DECK.length === 1 &&
			output.action === 'foreign-aid' &&
			output.byWhom === 'bot1'
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  we get the right parameters passed in for onCounterAction`);
	},
	checkParameters4: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'assassin';
		player.bot1.card2 = 'captain';
		player.bot1.coins = 1;
		player.bot2.card1 = 'captain';
		player.bot2.coins = 3;
		player.bot3.card1 = 'duke';

		const bots = makeBots();
		let output;
		bots.bot1.onTurn = () => ({ action: 'foreign-aid' });
		bots.bot2.onCounterAction = () => 'duke';
		bots.bot3.onCounterActionRound = (param) => {
			output = param;
			return false;
		};

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['contessa'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === 'assassin' &&
			game.PLAYER.bot1.card2 === 'captain' &&
			game.PLAYER.bot1.coins === 1 &&
			game.PLAYER.bot2.card1 === 'captain' &&
			game.PLAYER.bot2.coins === 3 &&
			game.PLAYER.bot3.card1 === 'duke' &&
			game.DECK.length === 1 &&
			output.action === 'foreign-aid' &&
			output.byWhom === 'bot1' &&
			output.counterer === 'bot2' &&
			output.card === 'duke'
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  we get the right parameters passed in for onCounterActionRound for duking`);
	},
	checkParameters5: async () => {
		const game = new COUP();

		const player = makePlayer();
		player.bot1.card1 = 'assassin';
		player.bot1.card2 = 'captain';
		player.bot1.coins = 4;
		player.bot2.card1 = 'captain';
		player.bot2.coins = 3;
		player.bot3.card1 = 'duke';

		const bots = makeBots();
		let output;
		bots.bot1.onTurn = () => ({ action: 'assassination', against: 'bot2' });
		bots.bot2.onCounterAction = () => 'contessa';
		bots.bot3.onCounterActionRound = (param) => {
			output = param;
			return false;
		};

		game.HISTORY = [];
		game.DISCARDPILE = [];
		game.BOTS = bots;
		game.PLAYER = player;
		game.DECK = ['contessa'];
		game.TURN = 2;
		game.whoIsLeft = () => ['bot1'];

		await game.turn();

		let status = style.red('FAIL');
		if (
			game.PLAYER.bot1.card1 === 'assassin' &&
			game.PLAYER.bot1.card2 === 'captain' &&
			game.PLAYER.bot1.coins === 1 &&
			game.PLAYER.bot2.card1 === 'captain' &&
			game.PLAYER.bot2.coins === 3 &&
			game.PLAYER.bot3.card1 === 'duke' &&
			game.DECK.length === 1 &&
			output.action === 'assassination' &&
			output.byWhom === 'bot1' &&
			output.toWhom === 'bot2' &&
			output.counterer === 'bot2' &&
			output.card === 'contessa'
		) {
			status = style.green('PASS');
		} else {
			pass = false;
		}
		console.info(`${status}  we get the right parameters passed in for onCounterActionRound for assassination`);
	},
};

console.info(`
 ████████╗ ███████╗ ███████╗ ████████╗ ██╗ ███╗   ██╗  ██████╗
 ╚══██╔══╝ ██╔════╝ ██╔════╝ ╚══██╔══╝ ██║ ████╗  ██║ ██╔════╝
    ██║    █████╗   ███████╗    ██║    ██║ ██╔██╗ ██║ ██║  ███╗
    ██║    ██╔══╝   ╚════██║    ██║    ██║ ██║╚██╗██║ ██║   ██║
    ██║    ███████╗ ███████║    ██║    ██║ ██║ ╚████║ ╚██████╔╝
    ╚═╝    ╚══════╝ ╚══════╝    ╚═╝    ╚═╝ ╚═╝  ╚═══╝  ╚═════╝
`);

Object.entries(TEST).forEach(async ([name, test]) => await test());

const ExitHandler = (exiting, error) => {
	if (error && error !== 1) {
		console.error(error);
	}

	console.info();

	if (!pass) {
		process.exit(1);
	} else {
		//now exit with a smile :)
		process.exit(0);
	}
};

process.on('exit', ExitHandler);
process.on('SIGINT', ExitHandler);
process.on('uncaughtException', ExitHandler);
