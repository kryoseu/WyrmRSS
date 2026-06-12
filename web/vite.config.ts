import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  envPrefix: "WYRM_",
  server: {
    proxy: {
      "/api": "http://localhost:3001",
    },
  },
  preview: {
    proxy: {
      "/api": process.env.WYRM_BACKEND_URL ?? "http://localhost:3001",
    },
  },
})
