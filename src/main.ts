import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import { useAuthStore } from './store/auth'

const pinia = createPinia()
const app = createApp(App)

app.use(pinia)
app.mount('#app')

useAuthStore(pinia).init()
