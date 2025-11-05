import './app.css';
import { createApp } from 'vue';
import { createRouter, createWebHistory } from 'vue-router';
import App from './App.vue';
import Home from './Home.vue';
import Features from './Features.vue';
import ApiDemo from './ApiDemo.vue';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: Home },
    { path: '/features', component: Features },
    { path: '/api-demo', component: ApiDemo }
  ]
});

createApp(App).use(router).mount('#app');
