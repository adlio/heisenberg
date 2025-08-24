// Sample SPA JavaScript
console.log('Sample SPA loaded');

document.addEventListener('DOMContentLoaded', function() {
    const app = document.getElementById('app');
    if (app) {
        console.log('App element found');
        
        // Add some interactivity
        const button = document.createElement('button');
        button.textContent = 'Click me!';
        button.onclick = function() {
            alert('Hello from Sample SPA!');
        };
        app.appendChild(button);
    }
});
