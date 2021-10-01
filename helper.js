const style = {
	parse: (text, start, end = '39m') => {
		if (text !== undefined) {
			return `\u001B[${start}${text}\u001b[${end}`;
		} else {
			return ``;
		}
	},
	black: (text) => style.parse(text, '30m'),
	red: (text) => style.parse(text, '31m'),
	green: (text) => style.parse(text, '32m'),
	yellow: (text) => style.parse(text, '33m'),
	blue: (text) => style.parse(text, '34m'),
	magenta: (text) => style.parse(text, '35m'),
	cyan: (text) => style.parse(text, '36m'),
	white: (text) => style.parse(text, '37m'),
	gray: (text) => style.parse(text, '90m'),
	bold: (text) => style.parse(text, '1m', '22m'),
};

module.exports = exports = {
	style,
};
