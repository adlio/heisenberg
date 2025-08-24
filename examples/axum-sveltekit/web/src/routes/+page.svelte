<script>
	import { onMount } from 'svelte';

	let todos = $state([]);
	let newTodo = $state('');
	let loading = $state(false);

	onMount(async () => {
		await loadTodos();
	});

	async function loadTodos() {
		loading = true;
		try {
			const response = await fetch('/api/todos');
			todos = await response.json();
		} catch (error) {
			console.error('Failed to load todos:', error);
		} finally {
			loading = false;
		}
	}

	async function addTodo() {
		if (!newTodo.trim()) return;
		
		try {
			const response = await fetch('/api/todos', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ title: newTodo.trim() })
			});
			const todo = await response.json();
			todos = [...todos, todo];
			newTodo = '';
		} catch (error) {
			console.error('Failed to add todo:', error);
		}
	}

	async function toggleTodo(id) {
		try {
			const response = await fetch(`/api/todos/${id}/toggle`, {
				method: 'POST'
			});
			const updatedTodo = await response.json();
			todos = todos.map(t => t.id === id ? updatedTodo : t);
		} catch (error) {
			console.error('Failed to toggle todo:', error);
		}
	}
</script>

<svelte:head>
	<title>Heisenberg + SvelteKit Todo</title>
</svelte:head>

<main>
	<h1>🚀 Heisenberg + SvelteKit</h1>
	<p>A minimal todo app demonstrating dual-mode serving</p>

	<div class="todo-form">
		<input 
			bind:value={newTodo} 
			placeholder="Add a new todo..." 
			onkeydown={(e) => e.key === 'Enter' && addTodo()}
		/>
		<button onclick={addTodo} disabled={!newTodo.trim()}>Add</button>
	</div>

	{#if loading}
		<p>Loading todos...</p>
	{:else if todos.length === 0}
		<p>No todos yet. Add one above!</p>
	{:else}
		<ul class="todo-list">
			{#each todos as todo (todo.id)}
				<li class:completed={todo.completed}>
					<button onclick={() => toggleTodo(todo.id)}>
						{todo.completed ? '✅' : '⭕'}
					</button>
					<span>{todo.title}</span>
				</li>
			{/each}
		</ul>
	{/if}

	<div class="info">
		<p><a href="/about">About this example</a></p>
		<p>Mode: <code>{import.meta.env.DEV ? 'Development (Proxy)' : 'Production (Embedded)'}</code></p>
	</div>
</main>

<style>
	main {
		max-width: 600px;
		margin: 2rem auto;
		padding: 1rem;
		font-family: system-ui, sans-serif;
	}

	.todo-form {
		display: flex;
		gap: 0.5rem;
		margin: 1rem 0;
	}

	.todo-form input {
		flex: 1;
		padding: 0.5rem;
		border: 1px solid #ccc;
		border-radius: 4px;
	}

	.todo-form button {
		padding: 0.5rem 1rem;
		background: #007acc;
		color: white;
		border: none;
		border-radius: 4px;
		cursor: pointer;
	}

	.todo-form button:disabled {
		background: #ccc;
		cursor: not-allowed;
	}

	.todo-list {
		list-style: none;
		padding: 0;
	}

	.todo-list li {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem;
		border-bottom: 1px solid #eee;
	}

	.todo-list li.completed span {
		text-decoration: line-through;
		opacity: 0.6;
	}

	.todo-list button {
		background: none;
		border: none;
		cursor: pointer;
		font-size: 1.2rem;
	}

	.info {
		margin-top: 2rem;
		padding-top: 1rem;
		border-top: 1px solid #eee;
		font-size: 0.9rem;
		color: #666;
	}

	code {
		background: #f5f5f5;
		padding: 0.2rem 0.4rem;
		border-radius: 3px;
		font-family: monospace;
	}
</style>