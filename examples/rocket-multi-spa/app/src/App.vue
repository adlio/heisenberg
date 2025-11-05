<template>
  <div>
    <h1>Main Application</h1>
    
    <nav>
      <router-link to="/">Home</router-link>
      <router-link to="/features">Features</router-link>
      <router-link to="/api-demo">API Demo</router-link>
      <a href="/admin/" class="admin-link">Admin →</a>
    </nav>

    <router-view />
  </div>
</template>

<script>
export default {
  name: 'App'
};
</script>
