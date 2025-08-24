<svelte:head>
	<title>About - Heisenberg + SvelteKit</title>
</svelte:head>

<main>
	<h1>About This Example</h1>
	
	<p><a href="/">← Back to Todo App</a></p>

	<section>
		<h2>🎯 What This Demonstrates</h2>
		<ul>
			<li><strong>Dual-mode serving</strong>: Development proxy vs production embedded assets</li>
			<li><strong>SvelteKit integration</strong>: Client-side routing with SPA fallback</li>
			<li><strong>API integration</strong>: Rust backend with JSON API endpoints</li>
			<li><strong>Modern SvelteKit</strong>: Using Svelte 5 runes and latest features</li>
		</ul>
	</section>

	<section>
		<h2>🔧 How It Works</h2>
		<p>In <strong>development mode</strong>:</p>
		<ul>
			<li>Heisenberg proxies frontend requests to Vite dev server (HMR enabled)</li>
			<li>API requests go directly to the Rust backend</li>
			<li>Fast iteration with hot reload</li>
		</ul>

		<p>In <strong>production mode</strong>:</p>
		<ul>
			<li>Frontend assets are embedded in the Rust binary</li>
			<li>Single binary deployment with no external dependencies</li>
			<li>SPA fallback handles client-side routing</li>
		</ul>
	</section>

	<section>
		<h2>🚀 Try It Yourself</h2>
		<pre><code># Development mode (proxy + HMR)
cargo run

# Production mode (embedded assets)
HEISENBERG_MODE=embed cargo run

# Build for production
cargo build --release</code></pre>
	</section>
</main><style>
	main {
		max-width: 800px;
		margin: 2rem auto;
		padding: 1rem;
		font-family: system-ui, sans-serif;
		line-height: 1.6;
	}

	section {
		margin: 2rem 0;
	}

	h2 {
		color: #333;
		border-bottom: 2px solid #007acc;
		padding-bottom: 0.5rem;
	}

	ul {
		padding-left: 1.5rem;
	}

	li {
		margin: 0.5rem 0;
	}

	pre {
		background: #f5f5f5;
		padding: 1rem;
		border-radius: 4px;
		overflow-x: auto;
	}

	code {
		font-family: 'Monaco', 'Menlo', monospace;
	}

	a {
		color: #007acc;
		text-decoration: none;
	}

	a:hover {
		text-decoration: underline;
	}
</style>