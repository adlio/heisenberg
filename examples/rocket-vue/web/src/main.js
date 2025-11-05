import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'
import Home from './Home.vue'
import About from './About.vue'
import ApiDemo from './ApiDemo.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: Home },
    { path: '/about', component: About },
    { path: '/api-demo', component: ApiDemo }
  ]
})

createApp(App).use(router).mount('#app')
