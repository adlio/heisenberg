<template>
  <div>
    <h1>
      Admin Dashboard
      <span class="admin-badge">ADMIN</span>
    </h1>
    
    <nav>
      <router-link to="/">Dashboard</router-link>
      <router-link to="/users">Users</router-link>
      <router-link to="/api-demo">API Demo</router-link>
      <a href="/" class="main-link">← Main App</a>
    </nav>

    <router-view />
  </div>
</template>

<script>
export default {
  name: 'App'
};
</script>
