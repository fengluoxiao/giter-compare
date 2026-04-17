import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import "./styles.css";
import { isGlassSupported, setLiquidGlassEffect, GlassMaterialVariant } from "tauri-plugin-liquid-glass-api";

// 启用液态玻璃效果（macOS 26+）
async function initLiquidGlass() {
  try {
    const supported = await isGlassSupported();
    if (supported) {
      await setLiquidGlassEffect({
        cornerRadius: 12,
        variant: GlassMaterialVariant.Sidebar,
      });
      document.body.classList.add("liquid-glass-enabled");
    }
  } catch (e) {
    console.log("Liquid glass not supported or failed:", e);
  }
}

initLiquidGlass();

createApp(App).use(router).mount("#app");
