const { ALLBOTS, CARDS, DECK, ACTIONS } = require('../constants.js');

class BOT {
	onTurn({ history, myCards, myCoins, otherPlayers, discardedCards }) {
		let action = ACTIONS()[Math.floor(Math.random() * ACTIONS().length)];
		const against = otherPlayers[Math.floor(Math.random() * otherPlayers.length)].name;

		if (myCoins > 10) {
			action = 'couping';
		}

		return {
			action,
			against,
		};
	}

	onChallengeActionRound({ history, myCards, myCoins, otherPlayers, discardedCards, action, byWhom, toWhom }) {
		return [true, false][Math.floor(Math.random() * 2)];
	}

	onCounterAction({ history, myCards, myCoins, otherPlayers, discardedCards, action, byWhom }) {
		if (action === 'assassination') {
			return [false, 'contessa'][Math.floor(Math.random() * 2)];
		} else if (action === 'stealing') {
			return [false, 'ambassador', 'captain'][Math.floor(Math.random() * 3)];
		}
	}

	onCounterActionRound({ history, myCards, myCoins, otherPlayers, discardedCards, action, byWhom, toWhom, card }) {
		return [true, false][Math.floor(Math.random() * 2)];
	}

	onSwappingCards({ history, myCards, myCoins, otherPlayers, discardedCards, newCards }) {
		return newCards;
	}

	onCardLoss({ history, myCards, myCoins, otherPlayers, discardedCards }) {
		return myCards[0];
	}
}

module.exports = exports = BOT;
