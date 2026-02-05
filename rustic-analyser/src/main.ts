import { createApp } from 'vue'
import { plugin, defaultConfig } from '@formkit/vue'
import './assets/css/style.css'
import App from './App.vue'
import router from './router'
import formkitConfig from './formkit.config'
import VChart from "vue-echarts";
import "echarts"; // pulls in everything

const app = createApp(App)
app.use(router)
app.use(plugin, defaultConfig(formkitConfig))
app.component("v-chart", VChart);

app.mount('#app')