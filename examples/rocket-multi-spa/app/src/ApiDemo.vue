<template>
  <div>
    <h2>API Demo</h2>
    <p>
      This page demonstrates API calls from Vue to the Rocket backend. 
      Click the button to make a request to <code>/api/hello</code>.
    </p>

    <div class="api-controls">
      <button @click="callAPI" :disabled="loading">
        Call API
      </button>
      <span v-if="callCount > 0">Requests made: {{ callCount }}</span>
    </div>

    <div v-if="response" class="response">
      <strong>Response:</strong>
      <pre>{{ JSON.stringify(response, null, 2) }}</pre>
    </div>

    <h2>Testing</h2>
    <ul>
      <li>Click the button to verify the proxy works</li>
      <li>Edit this file and save to test HMR</li>
      <li>Check the Network tab to see the API request</li>
      <li>Try the same from the <a href="/admin/">Admin Dashboard</a></li>
    </ul>
  </div>
</template>

<script>
export default {
  name: 'ApiDemo',
  data() {
    return {
      response: null,
      loading: false,
      callCount: 0
    };
  },
  methods: {
    async callAPI() {
      this.loading = true;
      try {
        const res = await fetch('/api/hello');
        const data = await res.json();
        this.response = data;
        this.callCount++;
      } catch (error) {
        this.response = { error: error.message };
      } finally {
        this.loading = false;
      }
    }
  }
};
</script>
