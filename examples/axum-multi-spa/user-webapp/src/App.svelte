<script>
  let currentPath = $state(window.location.pathname);
  let response = $state(null);
  let loading = $state(false);
  let callCount = $state(0);

  function navigate(path) {
    window.history.pushState({}, '', path);
    currentPath = path;
  }

  async function callAPI() {
    loading = true;
    try {
      const res = await fetch('/api/hello');
      const data = await res.json();
      response = data;
      callCount++;
    } catch (error) {
      response = { error: error.message };
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    const handlePopState = () => {
      currentPath = window.location.pathname;
    };
    window.addEventListener('popstate', handlePopState);
    return () => window.removeEventListener('popstate', handlePopState);
  });

  const mode = import.meta.env.MODE === 'production' ? 'production' : 'development';
</script>

<div>
  <h1>Main Application</h1>
  
  <nav>
    <a href="/" onclick={(e) => { e.preventDefault(); navigate('/'); }} class={currentPath === '/' ? 'active' : ''}>
      Home
    </a>
    <a href="/features" onclick={(e) => { e.preventDefault(); navigate('/features'); }} class={currentPath === '/features' ? 'active' : ''}>
      Features
    </a>
    <a href="/api-demo" onclick={(e) => { e.preventDefault(); navigate('/api-demo'); }} class={currentPath === '/api-demo' ? 'active' : ''}>
      API Demo
    </a>
    <a href="/admin/" style="margin-left: auto; color: #dc3545;">
      Admin →
    </a>
  </nav>

  {#if currentPath === '/'}
    <div>
      <p>Running in <code>{mode}</code> mode</p>

      {#if mode === 'development'}
        <h2>How It Works</h2>
        <p>
          Multiple servers are running: the Axum backend (port 3000) and two Vite dev servers 
          (app on port 5173, admin on port 5174). Heisenberg proxies requests between them:
        </p>
        <ul>
          <li>
            <a href="/api/hello" target="_blank">/api/hello</a> is served by the Axum backend
          </li>
          <li>
            Frontend assets for the main app are proxied to the Vite dev server on port 5173
          </li>
          <li>
            Frontend assets for <a href="/admin/">the admin app</a> are proxied to port 5174
          </li>
          <li>All other routes are handled by client-side routing in each SPA</li>
        </ul>
      {:else}
        <h2>Production Mode</h2>
        <p>
          The Axum backend serves pre-built static files from a single server. 
          API routes like <a href="/api/hello" target="_blank">/api/hello</a> are handled 
          by the backend, while all other routes serve the appropriate SPA.
        </p>
      {/if}

      <h2>Features to Test</h2>
      <ul>
        <li>
          <strong>Client-side Routing:</strong> Navigate between pages using the links above - 
          notice the instant navigation and URL changes
        </li>
        <li>
          <strong>Multi-SPA:</strong> Visit the <a href="/admin/">Admin Dashboard</a> - 
          it's a completely separate SPA with its own routing
        </li>
        <li>
          <strong>HMR:</strong> Edit this file and save - the page updates instantly 
          without losing state or refreshing
        </li>
        <li>
          <strong>API Proxy:</strong> Go to the API Demo page and click "Call API" - 
          the request goes through Heisenberg to the Axum backend
        </li>
        <li>
          <strong>Dual-mode:</strong> Build with <code>cargo run --release</code> to test 
          production mode with static file serving
        </li>
      </ul>
    </div>
  {:else if currentPath === '/features'}
    <div>
      <h2>Multi-SPA Architecture</h2>
      <p>
        This example demonstrates Heisenberg's ability to serve multiple independent 
        Single Page Applications from a single Rust backend.
      </p>

      <h2>How It Works</h2>
      <ol>
        <li>
          The main app (this one) is served at the root path <code>/</code>
        </li>
        <li>
          The admin app is served at <code>/admin/</code> with its own routing
        </li>
        <li>
          Each SPA has its own Vite dev server in development mode
        </li>
        <li>
          In production, both SPAs are built and served as static files
        </li>
        <li>
          API routes like <code>/api/*</code> are handled by the Axum backend
        </li>
      </ol>

      <h2>Benefits</h2>
      <ul>
        <li>Separate codebases for different parts of your application</li>
        <li>Different frameworks or versions for each SPA if needed</li>
        <li>Independent deployment and caching strategies</li>
        <li>Clear separation between public and admin interfaces</li>
      </ul>
    </div>
  {:else if currentPath === '/api-demo'}
    <div>
      <h2>API Demo</h2>
      <p>
        This page demonstrates API calls from Svelte to the Axum backend. 
        Click the button to make a request to <code>/api/hello</code>.
      </p>

      <div style="display: flex; align-items: center; gap: 15px; margin: 20px 0;">
        <button onclick={callAPI} disabled={loading}>
          Call API
        </button>
        {#if callCount > 0}
          <span>Requests made: {callCount}</span>
        {/if}
      </div>

      {#if response}
        <div class="response">
          <strong>Response:</strong>
          <pre>{JSON.stringify(response, null, 2)}</pre>
        </div>
      {/if}

      <h2>Testing</h2>
      <ul>
        <li>Click the button to verify the proxy works</li>
        <li>Edit this file and save to test HMR</li>
        <li>Check the Network tab to see the API request</li>
        <li>Try the same from the <a href="/admin/">Admin Dashboard</a></li>
      </ul>
    </div>
  {/if}
</div>
