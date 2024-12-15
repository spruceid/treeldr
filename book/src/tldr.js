function tldrLanguage(hljs) {
	return {
		name: 'TreeLDR',
		keywords: {
			keyword: 'prefix type fn let where',
			literal: 'true false null',
		},
		contains: [
			hljs.COMMENT('#'),
			// hljs.QUOTE_STRING_MODE,
			// hljs.NUMBER_MODE,
			// {
			// 	className: 'built_in',
			// 	begin: '\\b(print|input)\\b'
			// },
			{
				className: 'string',
				begin: '<',
				end: '>',
			},
			{
				className: 'string',
				begin: '"',
				end: '"',
			},
		],
	};
}

hljs.registerLanguage('tldr', tldrLanguage);
hljs.initHighlightingOnLoad();