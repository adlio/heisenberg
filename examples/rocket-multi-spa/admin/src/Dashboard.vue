<template>
  <div>
    <p>Running in <code>{{ mode }}</code> mode</p>

    <h2>Admin Interface</h2>
    <p>
      This is a completely separate Single Page Application from the main app. 
      It has its own routing, state management, and can even use a different 
      framework or version if needed.
    </p>

    <template v-if="mode === 'development'">
      <h2>Development Setup</h2>
      <p>
        The admin SPA runs on its own Vite dev server (port 3001), separate from 
        the main app (port 3000). Heisenberg routes requests to the correct dev server:
      </p>
      <ul>
        <li>Requests to <code>/admin/*</code> are proxied to port 3001</li>
        <li>Requests to <code>/</code> are proxied to port 3000</li>
        <li>API requests like <a href="/api/hello" target="_blank">/api/hello</a> go to the Rocket backend</li>
      </ul>
    </template>
    <template v-else>
      <h2>Production Mode</h2>
      <p>
        Both SPAs are built and served as static files from the Rocket backend. 
        The server intelligently routes requests to the correct SPA based on the URL path.
      </p>
    </template>

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
</template>

<script>
export default {
  name: 'Dashboard',
  computed: {
    mode() {
      return import.meta.env.MODE === 'production' ? 'production' : 'development';
    }
  }
};
</script>
