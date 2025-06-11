import { defineConfig } from 'vite'
import { viteSingleFile } from 'vite-plugin-singlefile'
import path from 'path'

export default defineConfig({
  plugins: [viteSingleFile()],
  build: {
    rollupOptions: {
      input: path.resolve(__dirname, 'docs/.vitepress/dist/index.html')
    },
    outDir: 'dist-single',
    emptyOutDir: true
  }
})
