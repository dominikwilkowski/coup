let { ALLBOTS, CARDS, DECK, ACTIONS } = require('./constants.js');
const { version } = require('./package.json');
const { style } = require('./helper.js');

// making clones so the bots don't break them
CARDS = CARDS();
DECK = DECK();
ACTIONS = ACTIONS();

class COUP {
	constructor() {
		// yes globals(sorta); sue me!
		this.HISTORY = [];
		this.DISCARDPILE = [];
		this.BOTS = {};
		this.PLAYER = {};
		this.DECK = DECK.slice(0);
		this.TURN = 0;
		this.ROUNDS = 0;
		this.ALLPLAYER = [];
	}

	play() {
		console.log(
			`\n\n` +
				`   ██████${style.yellow('╗')}  ██████${style.yellow('╗')}  ██${style.yellow('╗')}   ██${style.yellow(
					'╗'
				)} ██████${style.yellow('╗')}\n` +
				`  ██${style.yellow('╔════╝')} ██${style.yellow('╔═══')}██${style.yellow('╗')} ██${style.yellow(
					'║'
				)}   ██${style.yellow('║')} ██${style.yellow('╔══')}██${style.yellow('╗')}\n` +
				`  ██${style.yellow('║')}      ██${style.yellow('║')}   ██${style.yellow('║')} ██${style.yellow(
					'║'
				)}   ██${style.yellow('║')} ██████${style.yellow('╔╝')}\n` +
				`  ██${style.yellow('║')}      ██${style.yellow('║')}   ██${style.yellow('║')} ██${style.yellow(
					'║'
				)}   ██${style.yellow('║')} ██${style.yellow('╔═══╝')}\n` +
				`  ${style.yellow('╚')}██████${style.yellow('╗')} ${style.yellow('╚')}██████${style.yellow(
					'╔╝'
				)} ${style.yellow('╚')}██████${style.yellow('╔╝')} ██${style.yellow('║')}\n` +
				`   ${style.yellow('╚═════╝  ╚═════╝   ╚═════╝  ╚═╝')} v${version}\n` +
				`\n`
		);

		this.ALLPLAYER = this.getPlayer();
		this.getBots();
		this.makePlayers();
		this.handOutCards();
		this.electStarter();

		// this is the game loop
		return this.turn();
	}

	//////////////////////////////////////////////////////////////////////////////| Init methods

	// We need to make sure we don't play with more than 6 bots at a time
	// So here we make sure we shuffle all bots randomly and take 6 for this round
	getPlayer() {
		return ALLBOTS()
			.filter((item) => item !== undefined)
			.map((item) => [Math.random(), item])
			.sort((a, b) => a[0] - b[0])
			.map((item) => item[1])
			.slice(0, 6);
	}

	// we collect all players of this round and require them into memory
	// We also make sure each bot has all methods at least defined
	getBots(player) {
		try {
			this.ALLPLAYER.forEach((player) => {
				const bot = require(`./${player}/index.js`);
				this.BOTS[player] = new bot({ name: player });

				if (
					!this.BOTS[player].onTurn ||
					!this.BOTS[player].onChallengeActionRound ||
					!this.BOTS[player].onCounterAction ||
					!this.BOTS[player].onCounterActionRound ||
					!this.BOTS[player].onSwappingCards ||
					!this.BOTS[player].onCardLoss
				) {
					const missing = [
						'onTurn',
						'onChallengeActionRound',
						'onCounterAction',
						'onCounterActionRound',
						'onSwappingCards',
						'onCardLoss',
					].filter((method) => !Object.keys(this.BOTS[player]).includes(method));

					throw `🚨  ${style.red('The bot ')}${style.yellow(player)}${style.red(
						` is missing ${missing.length > 1 ? 'methods' : 'a method'}: `
					)}${style.yellow(missing.join(', '))}!\n`;
				}
			});
		} catch (error) {
			console.error(`Error in bot ${player}`);
			console.error(error);
			process.exit(1);
		}
	}

	// giving each player the right object so we can use them
	makePlayers(players) {
		players = this.shufflePlayer(this.ALLPLAYER);

		players.forEach((player) => {
			this.PLAYER[player] = {
				card1: undefined,
				card2: undefined,
				coins: 0,
			};
		});
	}

	shuffleCards() {
		this.DECK = this.DECK.filter((item) => item !== undefined)
			.map((item) => [Math.random(), item])
			.sort((a, b) => a[0] - b[0])
			.map((item) => item[1]);
	}

	// this shuffles all play of this round only
	shufflePlayer(player) {
		return player
			.filter((item) => item !== undefined)
			.map((item) => [Math.random(), item])
			.sort((a, b) => a[0] - b[0])
			.map((item) => item[1]);
	}

	handOutCards() {
		this.shuffleCards();

		Object.entries(this.PLAYER).forEach(([key, value]) => {
			this.PLAYER[key].card1 = this.DECK.pop();
			this.PLAYER[key].card2 = this.DECK.pop();
		});
	}

	electStarter() {
		this.TURN = Math.floor(Math.random() * Object.keys(this.PLAYER).length);
	}

	//////////////////////////////////////////////////////////////////////////////| Play methods

	turn() {
		const player = Object.keys(this.PLAYER)[this.goToNextPlayer()];

		let botAnswer;
		try {
			botAnswer = this.BOTS[player].onTurn(this.getGameState(player));
		} catch (error) {
			this.penalty(player, `the bot crashed`);
			console.error(`Error in bot ${player}`);
			console.error(error);
		}

		if (botAnswer && this.isValidTarget(botAnswer.action, botAnswer.against, player)) {
			const { action, against } = botAnswer;
			const playerAvatar = this.getAvatar(player);
			const targetAvatar = this.getAvatar(against);

			let skipAction = false;

			switch (action) {
				case 'taking-1':
					this.HISTORY.push({
						type: 'action',
						action: 'taking-1',
						from: player,
					});
					console.log(`🃏  ${playerAvatar} takes ${style.yellow('a coin')}`);
					break;
				case 'foreign-aid':
					this.HISTORY.push({
						type: 'action',
						action: 'foreign-aid',
						from: player,
					});
					console.log(`🃏  ${playerAvatar} takes 2 coins ${style.yellow('foreign aid')}`);
					break;
				case 'couping':
					this.HISTORY.push({
						type: 'action',
						action: 'couping',
						from: player,
						to: against,
					});
					console.log(`🃏  ${playerAvatar} coups ${targetAvatar}`);

					if (this.PLAYER[player].coins < 7) {
						this.penalty(player, `did't having enough coins for a coup`);
						skipAction = true;
					}

					if (!this.stillAlive(against)) {
						this.penalty(player, `tried to coup a dead player`);
						skipAction = true;
					}
					break;
				case 'taking-3':
					this.HISTORY.push({
						type: 'action',
						action: 'taking-3',
						from: player,
					});
					console.log(`🃏  ${playerAvatar} takes 3 coins with the ${style.yellow('duke')}`);
					break;
				case 'assassination':
					this.HISTORY.push({
						type: 'action',
						action: 'assassination',
						from: player,
						to: against,
					});
					console.log(`🃏  ${playerAvatar} assassinates ${targetAvatar}`);

					if (this.PLAYER[player].coins < 3) {
						this.penalty(player, `did't have enough coins for an assassination`);
						skipAction = true;
					} else if (!this.stillAlive(against)) {
						this.penalty(player, `tried to assassinat a dead player`);
						skipAction = true;
					} else {
						this.PLAYER[player].coins -= 3;
					}
					break;
				case 'stealing':
					this.HISTORY.push({
						type: 'action',
						action: 'stealing',
						from: player,
						to: against,
					});

					if (!this.stillAlive(against)) {
						this.penalty(player, `tried to steal from a dead player`);
						skipAction = true;
					}

					console.log(`🃏  ${playerAvatar} steals from ${targetAvatar}`);
					break;
				case 'swapping':
					this.HISTORY.push({
						type: 'action',
						action: 'swapping',
						from: player,
					});
					console.log(`🃏  ${playerAvatar} swaps two cards with the ${style.yellow('ambassador')}`);
					break;
				default:
					this.HISTORY.push({
						type: 'penalty',
						from: player,
					});
					this.penalty(
						player,
						`of issuing an invalid action: "${style.yellow(action)}", allowed: ${style.yellow(ACTIONS.join(', '))}`
					);
					skipAction = true;
			}

			if (!skipAction) this.runChallenges({ player, action, target: against });
		}

		if (this.whoIsLeft().length > 1 && this.ROUNDS < 1000) {
			this.ROUNDS++;
			return this.turn();
		} else if (this.ROUNDS >= 1000) {
			console.log('The game was stopped because of an infinite loop');
			return this.whoIsLeft();
		} else {
			const winner = this.whoIsLeft()[0];
			console.log(`\nThe winner is ${this.getAvatar(winner)}\n`);
			return [winner];
		}
	}

	goToNextPlayer() {
		this.TURN++;

		if (this.TURN > Object.keys(this.PLAYER).length - 1) {
			this.TURN = 0;
		}

		if (
			this.PLAYER[Object.keys(this.PLAYER)[this.TURN]].card1 ||
			this.PLAYER[Object.keys(this.PLAYER)[this.TURN]].card2
		) {
			return this.TURN;
		} else {
			return this.goToNextPlayer();
		}
	}

	getCardFromDeck() {
		const newCard = this.DECK.pop();

		if (!newCard && this.DECK.length > 0) {
			return this.getCardFromDeck();
		} else {
			return newCard;
		}
	}

	exchangeCard(card) {
		this.DECK.push(card);
		this.shuffleCards();

		return this.getCardFromDeck();
	}

	swapCards({ chosenCards = [], newCards, player }) {
		let oldCards = [];
		if (this.PLAYER[player].card1) oldCards.push(this.PLAYER[player].card1);
		if (this.PLAYER[player].card2) oldCards.push(this.PLAYER[player].card2);

		let allCards = oldCards.slice(0);
		if (newCards[0]) allCards.push(newCards[0]);
		if (newCards[1]) allCards.push(newCards[1]);

		chosenCards = chosenCards.filter((card) => allCards.includes(card)).slice(0, oldCards.length);

		this.PLAYER[player].card1 = chosenCards[0];
		this.PLAYER[player].card2 = chosenCards[1];

		allCards
			.filter((card) => {
				if (card && card === chosenCards[0]) {
					chosenCards[0] = undefined;
					return false;
				}
				if (card && card === chosenCards[1]) {
					chosenCards[1] = undefined;
					return false;
				}
				return true;
			})
			.map((card) => this.DECK.push(card));

		this.shuffleCards();
	}

	stillAlive(player) {
		let cards = 0;
		if (this.PLAYER[player].card1) cards++;
		if (this.PLAYER[player].card2) cards++;

		return cards > 0;
	}

	whoIsLeft() {
		return Object.keys(this.PLAYER).filter((player) => this.PLAYER[player].card1 || this.PLAYER[player].card2);
	}

	getPlayerObjects(players, filter = '') {
		return players
			.filter((user) => user !== filter)
			.map((player) => {
				let cards = 0;
				if (this.PLAYER[player].card1) cards++;
				if (this.PLAYER[player].card2) cards++;

				return {
					name: player,
					coins: this.PLAYER[player].coins,
					cards,
				};
			});
	}

	getGameState(player) {
		return {
			history: this.HISTORY.slice(0),
			myCards: this.getPlayerCards(player),
			myCoins: this.PLAYER[player].coins,
			otherPlayers: this.getPlayerObjects(this.whoIsLeft(), player),
			discardedCards: this.DISCARDPILE.slice(0),
		};
	}

	isValidTarget(action, target, player) {
		const doesExist = Object.keys(this.PLAYER).includes(target);
		const isRequired = ['couping', 'assassination', 'stealing'].includes(action);
		const isValid = (isRequired && doesExist) || !isRequired;

		if (!isValid) {
			this.penalty(player, `the bot gave invalid target "${target}"`);
		}

		return isValid;
	}

	// We leave this here for debugging our code
	wait(time) {
		return new Promise((resolve) => setTimeout(resolve, time));
	}

	getAvatar(player) {
		if (!player) {
			return player;
		} else if (!this.ALLPLAYER.includes(player)) {
			return `[${style.yellow(`${player}`)} -not found-]`;
		} else {
			return (
				style.yellow(`[${player} `) +
				// `${ this.PLAYER[ player ].card1 ? `${ style.red( this.PLAYER[ player ].card1.substring( 0, 2 ) ) } ` : '' }` +
				// `${ this.PLAYER[ player ].card2 ? `${ style.red( this.PLAYER[ player ].card2.substring( 0, 2 ) ) } ` : '' }` +
				`${this.PLAYER[player].card1 ? style.red('♥') : ''}` +
				`${this.PLAYER[player].card2 ? style.red('♥') : ''}` +
				` ${style.yellow(`💰 ${this.PLAYER[player].coins}]`)}`
			);
		}
	}

	getPlayerCards(player) {
		const myCards = [];
		if (this.PLAYER[player].card1) myCards.push(this.PLAYER[player].card1);
		if (this.PLAYER[player].card2) myCards.push(this.PLAYER[player].card2);
		return myCards;
	}

	losePlayerCard(player, card) {
		let lost = '';

		if (this.PLAYER[player].card1 === card) {
			lost = this.PLAYER[player].card1;
			this.PLAYER[player].card1 = undefined;
		} else if (this.PLAYER[player].card2 === card) {
			lost = this.PLAYER[player].card2;
			this.PLAYER[player].card2 = undefined;
		}

		this.HISTORY.push({
			type: 'lost-card',
			player,
			lost,
		});

		this.DISCARDPILE.push(lost);

		let lives = 0;
		if (this.PLAYER[player].card1) lives++;
		if (this.PLAYER[player].card2) lives++;

		console.log(`${lives > 0 ? '💔' : '☠️'}  ${this.getAvatar(player)} has lost the ${style.yellow(lost)}`);
	}

	penalty(player, reason) {
		let penalty = '';

		let lostCard;
		try {
			lostCard = this.BOTS[player].onCardLoss(this.getGameState(player));
		} catch (error) {
			this.PLAYER[player].card1 = undefined;
			this.PLAYER[player].card2 = undefined;
			console.error(`Error in bot ${player}`);
			console.error(error);
		}

		const _validCard = [this.PLAYER[player].card1, this.PLAYER[player].card2].includes(lostCard) && lostCard;

		if ((_validCard && this.PLAYER[player].card1 === lostCard) || (!_validCard && this.PLAYER[player].card1)) {
			penalty = this.PLAYER[player].card1;
		} else if ((_validCard && this.PLAYER[player].card2 === lostCard) || (!_validCard && this.PLAYER[player].card2)) {
			penalty = this.PLAYER[player].card2;
		}

		console.log(`🚨  ${this.getAvatar(player)} was penalised because ${style.yellow(reason)}`);
		this.losePlayerCard(player, penalty);
	}

	resolveChallenge({ challenger, byWhom, card, action, type, target, counterer, challengee }) {
		const challengeTypes = {
			'challenge-round': 'onChallengeActionRound',
			'counter-round': 'onCounterActionRound',
		};

		let botAnswer;
		try {
			botAnswer = this.BOTS[challenger][challengeTypes[type]]({
				...this.getGameState(challenger),
				action,
				byWhom,
				toWhom: target,
				counterer,
				card,
			});
		} catch (error) {
			this.penalty(challenger, `the bot crashed`);
			console.error(`Error in bot ${challenger}`);
			console.error(error);
		}

		if (botAnswer) {
			const lying = this.PLAYER[challengee].card1 !== card && this.PLAYER[challengee].card2 !== card;

			this.HISTORY.push({
				type,
				challenger,
				challengee,
				action,
				lying,
			});

			console.log(`❓  ${this.getAvatar(challengee)} was challenged by ${this.getAvatar(challenger)}`);

			if (lying) {
				this.HISTORY.push({
					type: 'penalty',
					player: challengee,
				});

				this.penalty(challengee, 'of lying');

				return true;
			} else {
				this.HISTORY.push({
					type: 'penalty',
					from: challenger,
				});

				this.penalty(challenger, `of challenging ${this.getAvatar(challengee)} unsuccessfully`);
				const newCard = this.exchangeCard(card);

				if (this.PLAYER[challengee].card1 === card) this.PLAYER[challengee].card1 = newCard;
				else if (this.PLAYER[challengee].card2 === card) this.PLAYER[challengee].card2 = newCard;

				this.HISTORY.push({
					type: 'unsuccessful-challenge',
					action: 'swap-1',
					from: challengee,
					card: card,
				});
				console.log(
					`↬  ${this.getAvatar(challengee)} put the ${style.yellow(card)} back in the deck and drew a new card`
				);

				return 'done';
			}
		}

		return false;
	}

	challengeRound({ player, target, card, action, type, counterer }) {
		let _hasBeenChallenged = false;

		const challengee = type === 'counter-round' ? counterer : player;

		Object.keys(this.PLAYER)
			.filter(
				(challenger) => challenger !== challengee && (this.PLAYER[challenger].card1 || this.PLAYER[challenger].card2)
			)
			.some((challenger) => {
				_hasBeenChallenged = this.resolveChallenge({
					challenger,
					byWhom: player,
					card,
					action,
					type,
					target,
					counterer,
					challengee,
				});
				return _hasBeenChallenged === 'done' ? true : _hasBeenChallenged;
			});

		return _hasBeenChallenged;
	}

	counterAction({ player, action, target }) {
		const actions = {
			'foreign-aid': ['duke', false],
			assassination: ['contessa', false],
			stealing: ['captain', 'ambassador', false],
		};
		const counter = {};
		if (action !== 'foreign-aid') {
			try {
				counter.counterAction = this.BOTS[target].onCounterAction({
					...this.getGameState(target),
					action,
					byWhom: player,
					toWhom: target,
				});
				counter.counterer = target;
			} catch (error) {
				this.penalty(target, `the bot crashed`);
				console.error(`Error in bot ${target}`);
				console.error(error);
			}
		} else {
			// Foreign aid. everyone gets a go!
			Object.keys(this.PLAYER)
				.filter((counterer) => counterer !== player && (this.PLAYER[counterer].card1 || this.PLAYER[counterer].card2))
				.some((counterer) => {
					let _hasBeenChallenged;
					try {
						_hasBeenChallenged = this.BOTS[counterer].onCounterAction({
							...this.getGameState(counterer),
							action,
							byWhom: player,
							toWhom: undefined,
						});
					} catch (error) {
						this.penalty(counterer, `the bot crashed`);
						console.error(`Error in bot ${counterer}`);
						console.error(error);
					}

					if (_hasBeenChallenged) {
						counter.counterAction = _hasBeenChallenged;
						counter.counterer = counterer;
						return true;
					}
				});
		}

		if (counter.counterAction) {
			if (!actions[action].includes(counter.counterAction)) {
				this.penalty(
					counter.counterer,
					`did't give a valid counter action ${style.yellow(counter.counterAction)} for ${style.yellow(action)}`
				);
				return true;
			}

			this.HISTORY.push({
				type: 'counter-action',
				action,
				from: player,
				to: target,
				counter: counter.counterAction,
				counterer: counter.counterer,
			});
			console.log(
				`❓  ${this.getAvatar(player)} was counter actioned by ${this.getAvatar(counter.counterer)} with ${style.yellow(
					counter.counterAction
				)}`
			);
			const _hasBeenChallenged = this.challengeRound({
				player,
				target,
				card: counter.counterAction,
				action,
				type: 'counter-round',
				counterer: counter.counterer,
			});
			return _hasBeenChallenged === 'done' ? true : !_hasBeenChallenged;
		}

		return false;
	}

	runChallenges({ action, player, target }) {
		if (action === 'taking-3' || action === 'assassination' || action === 'stealing' || action === 'swapping') {
			const card = {
				'taking-3': 'duke',
				assassination: 'assassin',
				stealing: 'captain',
				swapping: 'ambassador',
			}[action];

			const _hasBeenChallenged = this.challengeRound({
				player,
				card,
				action,
				type: 'challenge-round',
				target,
			});
			if (_hasBeenChallenged && _hasBeenChallenged !== 'done') {
				return;
			}
		}

		if (action === 'foreign-aid' || action === 'assassination' || action === 'stealing') {
			const _hasBeenChallenged = this.counterAction({ player, action, target });
			if (_hasBeenChallenged && _hasBeenChallenged !== 'done') {
				return;
			}
		}

		this.runActions({ player, action, target });
	}

	runActions({ player, action, target }) {
		if (!this.PLAYER[target] && !['taking-1', 'taking-3', 'swapping', 'foreign-aid'].includes(action)) {
			this.penalty(player, `did't give a valid (${target}) player`);
			return true;
		}

		if (!ACTIONS.includes(action)) {
			this.penalty(player, `did't give a valid (${action}) action`);
			return true;
		}

		if (this.PLAYER[player].coins > 10 && action !== 'couping') {
			this.penalty(player, `had too much coins and needed to coup`);
			return;
		}

		let disgarded;

		switch (action) {
			case 'taking-1':
				this.PLAYER[player].coins++;
				break;

			case 'foreign-aid':
				this.PLAYER[player].coins += 2;
				break;

			case 'couping':
				this.PLAYER[player].coins -= 7;
				try {
					disgarded = this.BOTS[target].onCardLoss(this.getGameState(target));
				} catch (error) {
					this.PLAYER[target].card1 = undefined;
					this.PLAYER[target].card2 = undefined;
					console.error(`Error in bot ${target}`);
					console.error(error);
				}

				if (this.PLAYER[target].card1 === disgarded && disgarded) {
					this.losePlayerCard(target, disgarded);
				} else if (this.PLAYER[target].card2 === disgarded && disgarded) {
					this.losePlayerCard(target, disgarded);
				} else {
					this.penalty(target, `did't give up a valid card "${disgarded}"`);
				}
				break;

			case 'taking-3':
				this.PLAYER[player].coins += 3;
				break;

			case 'assassination':
				try {
					disgarded = this.BOTS[target].onCardLoss(this.getGameState(target));
				} catch (error) {
					this.PLAYER[target].card1 = undefined;
					this.PLAYER[target].card2 = undefined;
					console.error(`Error in bot ${target}`);
					console.error(error);
				}

				if (this.PLAYER[target].card1 === disgarded && disgarded) {
					this.losePlayerCard(target, disgarded);
				} else if (this.PLAYER[target].card2 === disgarded && disgarded) {
					this.losePlayerCard(target, disgarded);
				} else {
					this.penalty(target, `did't give up a valid card "${disgarded}"`);
				}
				break;

			case 'stealing':
				if (this.PLAYER[target].coins < 2) {
					this.PLAYER[player].coins += this.PLAYER[target].coins;
					this.PLAYER[target].coins = 0;
				} else {
					this.PLAYER[player].coins += 2;
					this.PLAYER[target].coins -= 2;
				}
				break;

			case 'swapping':
				const newCards = [this.getCardFromDeck(), this.getCardFromDeck()];
				let chosenCards;
				try {
					chosenCards = this.BOTS[player].onSwappingCards({
						...this.getGameState(player),
						newCards: newCards.slice(0),
					});
				} catch (error) {
					this.penalty(player, `the bot crashed`);
					console.error(`Error in bot ${player}`);
					console.error(error);
				}

				this.swapCards({ chosenCards, player, newCards });
				break;
		}
	}
}

class LOOP {
	constructor() {
		this.DEBUG = false;
		this.WINNERS = {};
		this.LOG = '';
		this.ERROR = false;
		this.SCORE = {};
		this.ROUND = 0;
		this.ROUNDS = this.getRounds();

		ALLBOTS().forEach((player) => {
			this.WINNERS[player] = 0;
			this.SCORE[player] = 0;
		});
	}

	getScore(winners, allPlayer) {
		const winnerCount = winners.length;
		const loserCount = allPlayer.length - winnerCount;
		const loserScore = -1 / (allPlayer.length - 1);
		const winnerScore = ((loserScore * loserCount) / winnerCount) * -1;

		allPlayer.forEach((player) => {
			if (winners.includes(player)) {
				this.SCORE[player] += winnerScore;
			} else {
				this.SCORE[player] += loserScore;
			}
		});

		winners.forEach((player) => {
			this.WINNERS[player]++;
		});
	}

	displayScore(clear = false) {
		if (this.ROUND % 40 === 0 || this.ROUND === this.ROUNDS) {
			if (clear) process.stdout.write(`\u001b[${Object.keys(this.SCORE).length + 1}A\u001b[2K`);

			const done = String(Math.floor((this.ROUND / this.ROUNDS) * 100));
			const scoreWidth = Math.round(Math.log10(this.ROUNDS) + 8);
			process.stdout.write(`\u001b[2K${done.padEnd(3)}% done\n`);

			Object.keys(this.SCORE)
				.sort((a, b) => this.SCORE[b] - this.SCORE[a])
				.forEach((player) => {
					const percentage = this.ROUND > 0 ? `${((this.WINNERS[player] * 100) / this.ROUND).toFixed(3)}%` : '-';

					process.stdout.write(
						`\u001b[2K${style.gray(percentage.padStart(8))} ` +
							`${style.red(
								String(this.SCORE[player].toFixed(2))
									.padStart(scoreWidth - 2)
									.padEnd(scoreWidth)
							)} ` +
							`${style.yellow(player)}\n`
					);
				});
		}
	}

	getRounds() {
		const rIdx = process.argv.indexOf('-r');
		if (rIdx > 0 && process.argv.length > rIdx && Number.parseInt(process.argv[rIdx + 1]) > 0) {
			return Number.parseInt(process.argv[rIdx + 1]);
		}
		return 1000;
	}

	play() {
		let game = new COUP();
		const winners = game.play();
		this.ROUND++;

		this.displayScore(true);

		if (!winners || this.ERROR || this.LOG.includes(`STOP`)) {
			console.info(this.LOG);
			// console.info( JSON.stringify( game.HISTORY, null, 2 ) );
			this.ROUND = this.ROUNDS;
		}

		this.getScore(winners, game.ALLPLAYER);

		this.LOG = '';

		if (this.ROUND < this.ROUNDS) {
			// We run on next tick so the GC can get to work.
			// Otherwise it will work up a large memory footprint
			// when running over 100,000 games
			// (cause loops won't let the GC run efficiently)
			process.nextTick(() => this.play());
		} else {
			console.info();
		}
	}

	run(debug = false) {
		this.DEBUG = debug;

		console.log = (text) => {
			this.LOG += `${text}\n`;
		};
		console.error = (text) => {
			if (this.DEBUG) {
				this.ERROR = true;
				this.LOG += style.red(`🛑  ${text}\n`);
			}
		};
		console.info(`\nGame round started`);
		console.info('\n🎉   WINNERS  🎉\n');

		this.displayScore(false);

		this.play();
	}
}

module.exports = exports = {
	COUP,
	LOOP,
};
