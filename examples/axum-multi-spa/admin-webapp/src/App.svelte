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
  <h1>
    Admin Dashboard
    <span class="admin-badge">ADMIN</span>
  </h1>
  
  <nav>
    <a href="/admin/" onclick={(e) => { e.preventDefault(); navigate('/admin/'); }} class={currentPath === '/admin/' || currentPath === '/admin' ? 'active' : ''}>
      Dashboard
    </a>
    <a href="/admin/users" onclick={(e) => { e.preventDefault(); navigate('/admin/users'); }} class={currentPath === '/admin/users' ? 'active' : ''}>
      Users
    </a>
    <a href="/admin/api-demo" onclick={(e) => { e.preventDefault(); navigate('/admin/api-demo'); }} class={currentPath === '/admin/api-demo' ? 'active' : ''}>
      API Demo
    </a>
    <a href="/" style="margin-left: auto; color: #0066cc;">
      ← Main App
    </a>
  </nav>

  {#if currentPath === '/admin/' || currentPath === '/admin'}
    <div>
      <p>Running in <code>{mode}</code> mode</p>

      <h2>Admin Interface</h2>
      <p>
        This is a completely separate Single Page Application from the main app. 
        It has its own routing, state management, and can even use a different 
        framework or version if needed.
      </p>

      {#if mode === 'development'}
        <h2>Development Setup</h2>
        <p>
          The admin SPA runs on its own Vite dev server (port 5174), separate from 
          the main app (port 5173). Heisenberg routes requests to the correct dev server:
        </p>
        <ul>
          <li>Requests to <code>/admin/*</code> are proxied to port 5174</li>
          <li>Requests to <code>/</code> are proxied to port 5173</li>
          <li>API requests like <a href="/api/hello" target="_blank">/api/hello</a> go to the Axum backend</li>
        </ul>
      {:else}
        <h2>Production Mode</h2>
        <p>
          Both SPAs are built and served as static files from the Axum backend. 
          The server intelligently routes requests to the correct SPA based on the URL path.
        </p>
      {/if}

      <h2>Key Features</h2>
      <ul>
        <li>
          <strong>Separate Routing:</strong> The admin app has its own routes that don't 
          conflict with the main app
        </li>
        <li>
          <strong>Independent State:</strong> State in this SPA is completely separate 
          from the main app
        </li>
        <li>
          <strong>Shared Backend:</strong> Both SPAs can call the same API endpoints
        </li>
        <li>
          <strong>Hot Reload:</strong> Edit admin files and see changes instantly without 
          affecting the main app
        </li>
      </ul>
    </div>
  {:else if currentPath === '/admin/users'}
    <div>
      <h2>User Management</h2>
      <p>
        This page demonstrates client-side routing within the admin SPA. 
        Try refreshing the page - it should still work in both development and production modes.
      </p>

      <h2>Multi-SPA Benefits</h2>
      <ul>
        <li>
          <strong>Code Separation:</strong> Admin code is completely separate from the main app, 
          making it easier to maintain and secure
        </li>
        <li>
          <strong>Access Control:</strong> You can add authentication middleware specifically 
          for the <code>/admin/*</code> routes
        </li>
        <li>
          <strong>Different Tech Stacks:</strong> The admin panel could use a different framework 
          or library version than the main app
        </li>
        <li>
          <strong>Independent Deployment:</strong> Update the admin panel without touching 
          the main app (and vice versa)
        </li>
      </ul>
    </div>
  {:else if currentPath === '/admin/api-demo'}
    <div>
      <h2>Admin API Demo</h2>
      <p>
        This demonstrates that the admin SPA can make API calls to the same backend 
        as the main app. Click the button to call <code>/api/hello</code>.
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
        <li>Click the button to verify the API proxy works from the admin SPA</li>
        <li>Compare with the <a href="/">main app's API demo</a> - same backend, different SPAs</li>
        <li>Edit this file and save to test HMR in the admin SPA</li>
        <li>Check the Network tab to see the API request</li>
      </ul>
    </div>
  {/if}
</div>
