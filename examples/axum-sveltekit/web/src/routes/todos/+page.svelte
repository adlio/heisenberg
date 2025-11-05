<script>
	import { onMount } from 'svelte';

	/** @type {Array<{id: number, title: string, completed: boolean}>} */
	let todos = $state([]);
	let newTodo = $state('');
	let loading = $state(false);
	let addCount = $state(0);

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
			addCount++;
		} catch (error) {
			console.error('Failed to add todo:', error);
		}
	}

	/**
	 * @param {number} id
	 */
	async function toggleTodo(id) {
		try {
			const response = await fetch(`/api/todos/${id}/toggle`, {
				method: 'POST'
			});
			const updatedTodo = await response.json();
			todos = todos.map((t) => (t.id === id ? updatedTodo : t));
		} catch (error) {
			console.error('Failed to toggle todo:', error);
		}
	}
</script>

<div>
	<h1>Todos</h1>

	<p>
		This page demonstrates server-side state management with the Axum backend. Add todos and
		toggle their completion status - all data is stored in memory on the server.
	</p>

	<div style="display: flex; gap: 10px; margin: 20px 0;">
		<input
			type="text"
			bind:value={newTodo}
			placeholder="Add a new todo..."
			onkeydown={(e) => e.key === 'Enter' && addTodo()}
			style="flex: 1;"
		/>
		<button onclick={addTodo} disabled={!newTodo.trim()}>Add Todo</button>
	</div>

	{#if addCount > 0}
		<p style="color: #666; font-size: 14px;">Todos added: {addCount}</p>
	{/if}

	{#if loading}
		<p>Loading todos...</p>
	{:else if todos.length === 0}
		<div class="response">
			<p style="margin: 0;">No todos yet. Add one above to get started!</p>
		</div>
	{:else}
		<ul class="todo-list">
			{#each todos as todo (todo.id)}
				<li>
					<label class="checkbox">
						<input
							type="checkbox"
							checked={todo.completed}
							onchange={() => toggleTodo(todo.id)}
						/>
						<span class:completed={todo.completed}>{todo.title}</span>
					</label>
				</li>
			{/each}
		</ul>
	{/if}

	<h2>Testing</h2>
	<ul>
		<li>Add todos to verify the API works</li>
		<li>Toggle completion status by clicking the checkbox</li>
		<li>Edit this file and save to test HMR - your todos will remain</li>
		<li>Check the Network tab to see API requests</li>
		<li>Restart the server to see that todos are stored in memory (they'll be cleared)</li>
	</ul>
</div>

<style>
	.todo-list {
		list-style: none;
		padding: 0;
	}

	.todo-list li {
		padding: 10px;
		border-bottom: 1px solid #eee;
	}

	.checkbox {
		display: flex;
		align-items: center;
		gap: 10px;
		cursor: pointer;
	}

	.checkbox input {
		width: 18px;
		height: 18px;
		cursor: pointer;
	}

	.completed {
		text-decoration: line-through;
		opacity: 0.6;
	}
</style>
