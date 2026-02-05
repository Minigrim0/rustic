import { createApp } from 'vue'
import './assets/css/style.css'
import App from './App.vue'
import router from './router'
import VChart from "vue-echarts";
import "echarts"; // pulls in everything

const app = createApp(App)
app.use(router)
app.component("v-chart", VChart); 

app.mount('#app')