<template>
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
        API routes like <code>/api/*</code> are handled by the Rocket backend
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
</template>

<script>
export default {
  name: 'Features'
};
</script>
