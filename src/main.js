import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './styles/variables.css'
import './styles/global.css'
import './styles/reader.css'
import './styles/highlight.css'
import './styles/mobile.css'

createApp(App).use(createPinia()).mount('#app')
