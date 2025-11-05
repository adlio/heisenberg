<template>
  <div>
    <p>Running in <code>{{ mode }}</code> mode</p>

    <template v-if="mode === 'development'">
      <h2>How It Works</h2>
      <p>
        Multiple servers are running: the Rocket backend (port 8000) and two Vite dev servers 
        (app on port 3000, admin on port 3001). Heisenberg proxies requests between them:
      </p>
      <ul>
        <li>
          <a href="/api/hello" target="_blank">/api/hello</a> is served by the Rocket backend
        </li>
        <li>
          Frontend assets for the main app are proxied to the Vite dev server on port 3000
        </li>
        <li>
          Frontend assets for <router-link to="/admin/">the admin app</router-link> are proxied to port 3001
        </li>
        <li>All other routes are handled by client-side routing in each SPA</li>
      </ul>
    </template>
    <template v-else>
      <h2>Production Mode</h2>
      <p>
        The Rocket backend serves pre-built static files from a single server. 
        API routes like <a href="/api/hello" target="_blank">/api/hello</a> are handled 
        by the backend, while all other routes serve the appropriate SPA.
      </p>
    </template>

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
        <strong>API Proxy:</strong> Go to the <router-link to="/api-demo">API Demo</router-link> page and click "Call API" - 
        the request goes through Heisenberg to the Rocket backend
      </li>
      <li>
        <strong>Dual-mode:</strong> Build with <code>cargo run --release</code> to test 
        production mode with static file serving
      </li>
    </ul>
  </div>
</template>

<script>
export default {
  name: 'Home',
  computed: {
    mode() {
      return import.meta.env.MODE === 'production' ? 'production' : 'development';
    }
  }
};
</script>
