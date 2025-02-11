function tldrLanguage(hljs) {
	const COMPACT_IRI = {
		className: 'attr',
		begin: /[a-zA-Z0-9_]+:[a-zA-Z0-9_]*/,
	};

	return {
		name: 'TreeLDR',
		keywords: {
			keyword: 'prefix type let where all list',
			literal: 'true false null',
		},
		contains: [
			hljs.COMMENT('#'),
			COMPACT_IRI,
			{
				className: 'attr',
				begin: '<',
				end: '>',
			},
			{
				className: 'string',
				begin: '"',
				end: '"',
			},
			{
				className: 'attr',
				begin: '\'',
				end: '\'',
			},
		],
	};
}

hljs.registerLanguage('tldr', tldrLanguage);
hljs.initHighlightingOnLoad();