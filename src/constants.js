const path = require('path');
const fs = require('fs');

const getPlayer = (thisPath = './bots/') => {
	const allPlayer = fs
		.readdirSync(thisPath)
		.map((name) => path.join(process.cwd(), thisPath, name))
		.filter(
			(folder) =>
				fs.lstatSync(folder).isDirectory() &&
				!folder.endsWith('assets') &&
				!folder.startsWith('.') &&
				folder !== 'node_modules'
		)
		.map((name) => name.split('/').slice(-1)[0]);

	if (allPlayer.length < 2) {
		console.error(`\n🛑  We need at least two player to play this game!\n`);
		process.exit(1);
	} else {
		return allPlayer;
	}
};

const ALLBOTS = getPlayer;

// prettier-ignore
const CARDS = () => [
	'duke',
	'assassin',
	'captain',
	'ambassador',
	'contessa'
];

const getStack = (cards = CARDS()) => {
	let STACK = [];
	cards.forEach((card) => (STACK = [...STACK, ...new Array(3).fill(card)]));
	return STACK;
};

const DECK = getStack;

// prettier-ignore
const ACTIONS = () => [
	'taking-1',
	'foreign-aid',
	'couping',
	'taking-3',
	'assassination',
	'stealing',
	'swapping'
];

module.exports = exports = {
	ALLBOTS,
	CARDS,
	DECK,
	ACTIONS,
};
