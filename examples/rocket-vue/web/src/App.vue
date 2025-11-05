<template>
  <div class="container">
    <h1 class="vue-green">Rocket-Vue Example (DEV MODE)</h1>
    <p>This demonstrates Heisenberg's Rocket adapter with live Vue development.</p>
    
    <button @click="callAPI">Call API</button>
    <div v-if="response" class="response">
      <strong>API Response:</strong><br>
      {{ response }}
    </div>
    
    <h2>Features Demonstrated:</h2>
    <ul>
      <li>Live Vue development with HMR</li>
      <li>API proxy to Rocket backend</li>
      <li>Dual-mode operation (dev/prod)</li>
    </ul>
  </div>
</template>

<script>
export default {
  data() {
    return {
      response: null
    }
  },
  methods: {
    async callAPI() {
      try {
        const res = await fetch('/api/hello')
        this.response = await res.json()
      } catch (error) {
        this.response = { error: error.message }
      }
    }
  }
}
</script>

<style>
body { font-family: Arial, sans-serif; margin: 40px; }
.container { max-width: 600px; margin: 0 auto; }
button { 
  padding: 10px 20px; 
  margin: 10px 0; 
  cursor: pointer; 
  background: #42b883; 
  color: white; 
  border: none; 
  border-radius: 4px; 
}
button:hover { background: #369870; }
.response { 
  background: #f5f5f5; 
  padding: 15px; 
  margin: 10px 0; 
  border-radius: 5px; 
}
.vue-green { color: #42b883; }
</style>