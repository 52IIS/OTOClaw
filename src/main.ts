import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { MotionPlugin } from '@vueuse/motion'
import App from './App.vue'
import './styles/globals.css'
import './lib/logger'

console.log(
  '%c🦞 OTOClaw  启动',
  'background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; font-size: 16px; padding: 8px 16px; border-radius: 4px; font-weight: bold;'
);
console.log(
  '%c提示: 打开开发者工具 (Cmd+Option+I / Ctrl+Shift+I) 可以查看详细日志',
  'color: #888; font-size: 12px;'
);

const app = createApp(App)
app.use(createPinia())
app.use(MotionPlugin)
app.mount('#root')
