import './app.css';
import { createApp } from 'vue';
import { createRouter, createWebHistory } from 'vue-router';
import App from './App.vue';
import Dashboard from './Dashboard.vue';
import Users from './Users.vue';
import ApiDemo from './ApiDemo.vue';

const router = createRouter({
  history: createWebHistory('/admin/'),
  routes: [
    { path: '/', component: Dashboard },
    { path: '/users', component: Users },
    { path: '/api-demo', component: ApiDemo }
  ]
});

createApp(App).use(router).mount('#app');
