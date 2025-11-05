<script>
	import { page } from '$app/stores';

	let { children } = $props();
</script>

<svelte:head>
	<title>Heisenberg + SvelteKit</title>
</svelte:head>

<nav>
	<a href="/" class:active={$page.url.pathname === '/'}>Home</a>
	<a href="/about" class:active={$page.url.pathname === '/about'}>About</a>
	<a href="/todos" class:active={$page.url.pathname === '/todos'}>Todos</a>
</nav>

<div class="container">
	{@render children?.()}
</div>

<style>
	:global(body) {
		font-family: system-ui, sans-serif;
		line-height: 1.6;
		max-width: 800px;
		margin: 0 auto;
		padding: 20px;
	}

	nav {
		border-bottom: 1px solid #ddd;
		padding-bottom: 10px;
		margin-bottom: 20px;
	}

	nav a {
		margin-right: 15px;
		text-decoration: none;
		color: #0066cc;
	}

	nav a.active {
		font-weight: bold;
		text-decoration: underline;
	}

	:global(h1) {
		margin-top: 0;
	}

	:global(h2) {
		margin-top: 30px;
	}

	:global(button) {
		padding: 10px 20px;
		background: #0066cc;
		color: white;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		font-size: 16px;
	}

	:global(button:hover:not(:disabled)) {
		background: #0052a3;
	}

	:global(button:disabled) {
		opacity: 0.6;
		cursor: not-allowed;
	}

	:global(.response) {
		background: #f5f5f5;
		padding: 15px;
		margin: 20px 0;
		border-radius: 4px;
		border-left: 3px solid #0066cc;
	}

	:global(.response pre) {
		margin: 10px 0 0 0;
		overflow-x: auto;
	}

	:global(code) {
		background: #f5f5f5;
		padding: 2px 6px;
		border-radius: 3px;
	}

	:global(input[type="text"]) {
		padding: 10px;
		border: 1px solid #ddd;
		border-radius: 4px;
		font-size: 16px;
	}
</style>
