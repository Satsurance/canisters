import { defineConfig, loadEnv } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  // Load environment variables based on mode
  const env = loadEnv(mode, process.cwd(), '')

  // Determine if we're in local development
  const isDevelopment = mode === 'development'

  // Set API base URL based on mode
  // Development: use relative URLs with proxy
  // Production: use testnet.satsurance.xyz
  const apiBaseUrl = isDevelopment ? '' : 'https://testnet.satsurance.xyz'

  // Configure proxy for local development only
  // Note: This proxy only works during 'npm run dev', not in production builds
  const serverConfig = isDevelopment ? {
    proxy: {
      '/api': {
        // You can override this with VITE_API_PROXY environment variable
        target: env.VITE_API_PROXY || 'http://localhost:3050',
        changeOrigin: true,
        secure: false,
        rewrite: (path) => path.replace(/^\/api/, '')
      }
    }
  } : {}

  return {
    plugins: [vue()],
    build: {
      outDir: 'dist',
      emptyOutDir: true
    },
    base: './',
    server: serverConfig,
    define: {
      __API_BASE_URL__: JSON.stringify(apiBaseUrl)
    }
  }
})
