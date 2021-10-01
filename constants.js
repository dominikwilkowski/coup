const path = require('path');
const fs = require('fs');

const getPlayer = (thisPath = '.') => {
	const allPlayer = fs
		.readdirSync(thisPath)
		.map((name) => path.join(thisPath, name))
		.filter((item) => fs.lstatSync(item).isDirectory())
		.filter((folder) => !folder.startsWith('.') && folder !== 'node_modules');

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
