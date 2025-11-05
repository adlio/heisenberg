<template>
  <div>
    <h1>Home</h1>
    <p>Running in <code>{{ mode }}</code> mode</p>

    <template v-if="mode === 'development'">
      <h2>How It Works</h2>
      <p>
        Two servers are running: the Rocket backend (port 8000) and the Vite
        dev server (port 3000). Heisenberg proxies requests between them:
      </p>
      <ul>
        <li>
          <a href="/api/hello" target="_blank">/api/hello</a>
          is served by the Rocket backend
        </li>
        <li>
          Frontend assets like
          <a href="/src/main.js" target="_blank">/src/main.js</a>
          are proxied to the Vite dev server
        </li>
        <li>All other routes are handled by Vue Router</li>
      </ul>
    </template>

    <template v-else>
      <h2>Production Mode</h2>
      <p>
        The Rocket backend serves pre-built static files from a single
        server. API routes like
        <a href="/api/hello" target="_blank">/api/hello</a>
        are handled by the backend, while all other routes serve the Vue
        app.
      </p>
    </template>

    <h2>Features to Test</h2>
    <ul>
      <li>
        <strong>Vue Router:</strong> Navigate between pages using the links
        above - notice the instant navigation and URL changes
      </li>
      <li>
        <strong>HMR:</strong> Edit this file and save - the page updates
        instantly without losing state or refreshing
      </li>
      <li>
        <strong>API Proxy:</strong> Go to the API Demo page and click "Call
        API" - the request goes through Heisenberg to the Rocket backend
      </li>
      <li>
        <strong>Dual-mode:</strong> Build with
        <code>cargo run --release</code> to test production mode with static
        file serving
      </li>
    </ul>
  </div>
</template>

<script>
export default {
  data() {
    return {
      mode: import.meta.env.MODE === 'production' ? 'production' : 'development'
    }
  }
}
</script>
