const vscode = require("vscode");

function activate(context) {
	context.subscriptions.push(configureLanguage());
}

function deactivate() {}

module.exports = {
	activate,
	deactivate
};

/**
 * Sets up additional language configuration that's impossible to do via a
 * separate language-configuration.json file. See [1] for more information.
 *
 * [1]: https://github.com/Microsoft/vscode/issues/11514#issuecomment-244707076
 */
function configureLanguage() {
	return vscode.languages.setLanguageConfiguration('treeldr', {
		onEnterRules: [
			{
				// Doc single-line comment
				// e.g. ///|
				beforeText: /^\s*\/{3}.*$/,
				action: { indentAction: vscode.IndentAction.None, appendText: '/// ' },
			},
			{
				// Parent doc single-line comment
				// e.g. //!|
				beforeText: /^\s*\/{2}\!.*$/,
				action: { indentAction: vscode.IndentAction.None, appendText: '//! ' },
			}
		],
	});
}