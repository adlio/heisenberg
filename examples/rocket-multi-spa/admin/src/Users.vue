<template>
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
</template>

<script>
export default {
  name: 'Users'
};
</script>
