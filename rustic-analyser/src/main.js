import { createApp } from 'vue'
import './assets/css/style.css'
import App from './App.vue'

/**
 * Initialize the Vue application
 * This is the main entry point for the Rustic Analyser app
 */
const app = createApp(App)

// Mount the application to the DOM
app.mount('#app')

// Log app initialization
console.log('Rustic Analyser app initialized')