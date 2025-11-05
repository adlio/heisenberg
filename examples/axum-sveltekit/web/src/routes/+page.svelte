<script>
	const mode = import.meta.env.DEV ? 'development' : 'production';
</script>

<div>
	<h1>Home</h1>
	<p>Running in <code>{mode}</code> mode</p>

	{#if mode === 'development'}
		<h2>How It Works</h2>
		<p>
			Two servers are running: the Axum backend (port 3001) and the Vite dev server (port 5173).
			Heisenberg proxies requests between them:
		</p>
		<ul>
			<li>
				<a href="/api/todos" target="_blank">/api/todos</a> is served by the Axum backend
			</li>
			<li>Frontend assets are proxied to the Vite dev server with Hot Module Reload (HMR)</li>
			<li>All other routes are handled by SvelteKit's router</li>
		</ul>
	{:else}
		<h2>Production Mode</h2>
		<p>
			The Axum backend serves pre-built static files from a single server. API routes like
			<a href="/api/todos" target="_blank">/api/todos</a> are handled by the backend, while all
			other routes serve the SvelteKit app.
		</p>
	{/if}

	<h2>Features to Test</h2>
	<ul>
		<li>
			<strong>SvelteKit Router:</strong> Navigate between pages using the links above - notice the
			instant navigation and URL changes
		</li>
		<li>
			<strong>HMR:</strong> Edit this file and save - the page updates instantly without losing state
			or refreshing
		</li>
		<li>
			<strong>API Proxy:</strong> Go to the Todos page and add items - requests go through Heisenberg
			to the Axum backend
		</li>
		<li>
			<strong>Dual-mode:</strong> Build with <code>cargo run --release</code> to test production mode
			with static file serving
		</li>
	</ul>
</div>