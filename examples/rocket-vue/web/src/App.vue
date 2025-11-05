<template>
  <div>
    <nav>
      <router-link to="/" :class="{ active: $route.path === '/' }">Home</router-link>
      <router-link to="/about" :class="{ active: $route.path === '/about' }">About</router-link>
      <router-link to="/api-demo" :class="{ active: $route.path === '/api-demo' }">API Demo</router-link>
    </nav>
    <div class="container">
      <router-view />
    </div>
  </div>
</template>

<style>
body {
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

h1 {
  margin-top: 0;
}

h2 {
  margin-top: 30px;
}

button {
  padding: 10px 20px;
  background: #0066cc;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
}

button:hover:not(:disabled) {
  background: #0052a3;
}

button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.response {
  background: #f5f5f5;
  padding: 15px;
  margin: 20px 0;
  border-radius: 4px;
  border-left: 3px solid #0066cc;
}

.response pre {
  margin: 10px 0 0 0;
  overflow-x: auto;
}

code {
  background: #f5f5f5;
  padding: 2px 6px;
  border-radius: 3px;
}
</style>
