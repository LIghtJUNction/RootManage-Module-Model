import { defineConfig } from 'vite'
import { readFileSync, writeFileSync, readdirSync, statSync, existsSync } from 'fs'
import { join, extname, relative, dirname } from 'path'
import { createHash } from 'crypto'

// 递归复制VitePress构建输出并创建单文件HTML
function createSingleHTMLBundle() {
  return {
    name: 'create-single-html',
    apply: 'build',
    closeBundle() {
      const vitepressDistPath = 'docs/.vitepress/dist'
      const outputPath = 'dist-single'
      
      if (!existsSync(vitepressDistPath)) {
        console.error('❌ VitePress build output not found. Please run "npm run build" first.')
        return
      }

      try {
        // 读取主HTML文件
        const indexPath = join(vitepressDistPath, 'index.html')
        if (!existsSync(indexPath)) {
          console.error('❌ index.html not found in VitePress output')
          return
        }

        let htmlContent = readFileSync(indexPath, 'utf-8')

        // 内联所有CSS文件
        htmlContent = inlineAssets(htmlContent, vitepressDistPath, '.css', 'style')
        
        // 内联所有JavaScript文件
        htmlContent = inlineAssets(htmlContent, vitepressDistPath, '.js', 'script')
        
        // 内联所有图片和字体文件
        htmlContent = inlineImageAssets(htmlContent, vitepressDistPath)
        htmlContent = inlineFontAssets(htmlContent, vitepressDistPath)

        // 确保输出目录存在
        if (!existsSync(outputPath)) {
          require('fs').mkdirSync(outputPath, { recursive: true })
        }

        // 写入单文件HTML
        const outputFile = join(outputPath, 'index.html')
        writeFileSync(outputFile, htmlContent, 'utf-8')
        
        console.log('✅ Successfully created single HTML file:', outputFile)
        console.log(`📦 File size: ${(htmlContent.length / 1024).toFixed(2)} KB`)
        
      } catch (error) {
        console.error('❌ Error creating single HTML bundle:', error.message)
      }
    }
  }
}

// 内联CSS和JS资产
function inlineAssets(htmlContent, basePath, extension, tagType) {
  const assetRegex = extension === '.css' 
    ? /<link[^>]*rel="stylesheet"[^>]*href="([^"]+\.css)"[^>]*>/gi
    : /<script[^>]*src="([^"]+\.js)"[^>]*><\/script>/gi

  return htmlContent.replace(assetRegex, (match, assetPath) => {
    try {
      // 处理相对路径
      const fullPath = assetPath.startsWith('/') 
        ? join(basePath, assetPath.slice(1))
        : join(basePath, assetPath)
      
      if (existsSync(fullPath)) {
        const content = readFileSync(fullPath, 'utf-8')
        return tagType === 'style' 
          ? `<style>${content}</style>`
          : `<script>${content}</script>`
      } else {
        console.warn(`⚠️  Asset not found: ${fullPath}`)
        return match
      }
    } catch (error) {
      console.warn(`⚠️  Error inlining ${assetPath}:`, error.message)
      return match
    }
  })
}

// 内联图片资产
function inlineImageAssets(htmlContent, basePath) {
  const imageRegex = /<img[^>]*src="([^"]+\.(png|jpg|jpeg|gif|svg|ico))"[^>]*>/gi
  
  return htmlContent.replace(imageRegex, (match, imagePath) => {
    try {
      const fullPath = imagePath.startsWith('/') 
        ? join(basePath, imagePath.slice(1))
        : join(basePath, imagePath)
      
      if (existsSync(fullPath)) {
        const imageBuffer = readFileSync(fullPath)
        const base64 = imageBuffer.toString('base64')
        const ext = extname(imagePath).slice(1).toLowerCase()
        const mimeType = getMimeType(ext)
        const dataUri = `data:${mimeType};base64,${base64}`
        
        return match.replace(imagePath, dataUri)
      } else {
        console.warn(`⚠️  Image not found: ${fullPath}`)
        return match
      }
    } catch (error) {
      console.warn(`⚠️  Error inlining image ${imagePath}:`, error.message)
      return match
    }
  })
}

// 内联字体资产
function inlineFontAssets(htmlContent, basePath) {
  const fontRegex = /url\(['"]?([^'"]*\.(woff2?|ttf|eot|otf))['"]?\)/gi
  
  return htmlContent.replace(fontRegex, (match, fontPath) => {
    try {
      const fullPath = fontPath.startsWith('/') 
        ? join(basePath, fontPath.slice(1))
        : join(basePath, fontPath)
      
      if (existsSync(fullPath)) {
        const fontBuffer = readFileSync(fullPath)
        const base64 = fontBuffer.toString('base64')
        const ext = extname(fontPath).slice(1).toLowerCase()
        const mimeType = getFontMimeType(ext)
        const dataUri = `data:${mimeType};base64,${base64}`
        
        return `url(${dataUri})`
      } else {
        console.warn(`⚠️  Font not found: ${fullPath}`)
        return match
      }
    } catch (error) {
      console.warn(`⚠️  Error inlining font ${fontPath}:`, error.message)
      return match
    }
  })
}

// 获取MIME类型
function getMimeType(ext) {
  const mimeTypes = {
    'png': 'image/png',
    'jpg': 'image/jpeg',
    'jpeg': 'image/jpeg',
    'gif': 'image/gif',
    'svg': 'image/svg+xml',
    'ico': 'image/x-icon',
    'webp': 'image/webp'
  }
  return mimeTypes[ext] || 'application/octet-stream'
}

// 获取字体MIME类型
function getFontMimeType(ext) {
  const fontMimeTypes = {
    'woff': 'font/woff',
    'woff2': 'font/woff2',
    'ttf': 'font/ttf',
    'otf': 'font/otf',
    'eot': 'application/vnd.ms-fontobject'
  }
  return fontMimeTypes[ext] || 'application/octet-stream'
}

export default defineConfig({
  plugins: [createSingleHTMLBundle()],
  build: {
    // 这个配置主要是为了让Vite能够运行我们的插件
    // 实际的输入来自VitePress的构建输出
    rollupOptions: {
      input: 'bundle.js', // 虚拟入口点
      external: ['fs', 'path', 'crypto'] // Node.js内置模块
    }
  }
})