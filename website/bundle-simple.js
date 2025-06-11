import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

console.log('🚀 Starting bundle process...');

// 路径配置
const distDir = path.join(__dirname, 'docs/.vitepress/dist');
const htmlFile = path.join(distDir, 'index.html');
const outputDir = path.join(__dirname, 'dist-single');
const outputFile = path.join(outputDir, 'index.html');

// 确保输出目录存在
if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
}

// 读取 HTML 内容
let html = fs.readFileSync(htmlFile, 'utf-8');
console.log('📖 Reading HTML file...');

// 内联CSS文件
console.log('🎨 Inlining CSS...');
const cssRegex = /<link[^>]*rel=['"](preload )?stylesheet['"][^>]*href=['"]([^'"]*\.css)['"][^>]*>/g;
let cssMatch;
let cssCount = 0;
while ((cssMatch = cssRegex.exec(html)) !== null) {
    const cssPath = cssMatch[2];
    const fullCssPath = path.join(distDir, cssPath.startsWith('/') ? cssPath.slice(1) : cssPath);
    
    if (fs.existsSync(fullCssPath)) {
        const cssContent = fs.readFileSync(fullCssPath, 'utf-8');
        const styleTag = `<style>${cssContent}</style>`;
        html = html.replace(cssMatch[0], styleTag);
        cssCount++;
    }
}
console.log(`✅ Inlined ${cssCount} CSS files`);

// 内联JavaScript文件
console.log('⚡ Inlining JavaScript...');
const jsRegex = /<script[^>]*src=['"]([^'"]*\.js)['"][^>]*><\/script>/g;
let jsMatch;
let jsCount = 0;
while ((jsMatch = jsRegex.exec(html)) !== null) {
    const jsPath = jsMatch[1];
    const fullJsPath = path.join(distDir, jsPath.startsWith('/') ? jsPath.slice(1) : jsPath);
    
    if (fs.existsSync(fullJsPath)) {
        const jsContent = fs.readFileSync(fullJsPath, 'utf-8');
        const scriptTag = `<script>${jsContent}</script>`;
        html = html.replace(jsMatch[0], scriptTag);
        jsCount++;
    }
}
console.log(`✅ Inlined ${jsCount} JavaScript files`);

// 内联图片
console.log('🖼️ Inlining images...');
const imgRegex = /src=['"]\/assets\/([^'"]+\.(png|jpg|jpeg|gif|svg|ico))['"]|href=['"]\/assets\/([^'"]+\.(png|jpg|jpeg|gif|svg|ico))['"]|url\(['"]?\/assets\/([^'"]*\.(png|jpg|jpeg|gif|svg|ico))['"]?\)/g;
let imgMatch;
let imgCount = 0;
while ((imgMatch = imgRegex.exec(html)) !== null) {
    const assetPath = imgMatch[1] || imgMatch[3] || imgMatch[5];
    if (assetPath) {
        const fullAssetPath = path.join(distDir, 'assets', assetPath);
        if (fs.existsSync(fullAssetPath)) {
            try {
                const assetContent = fs.readFileSync(fullAssetPath);
                const ext = path.extname(assetPath).toLowerCase();
                const mimeType = {
                    '.png': 'image/png',
                    '.jpg': 'image/jpeg',
                    '.jpeg': 'image/jpeg',
                    '.gif': 'image/gif',
                    '.svg': 'image/svg+xml',
                    '.ico': 'image/x-icon'
                }[ext] || 'image/png';
                
                const base64 = assetContent.toString('base64');
                const dataUri = `data:${mimeType};base64,${base64}`;
                html = html.replace(imgMatch[0], imgMatch[0].replace(`/assets/${assetPath}`, dataUri));
                imgCount++;
            } catch (e) {
                console.warn(`⚠️ Could not encode ${assetPath}: ${e.message}`);
            }
        }
    }
}
console.log(`✅ Inlined ${imgCount} images`);

// 内联字体
console.log('🔤 Inlining fonts...');
const fontRegex = /url\(['"]?\/assets\/([^'"]*\.(woff2?|ttf|eot|otf))['"]?\)/g;
let fontMatch;
let fontCount = 0;
while ((fontMatch = fontRegex.exec(html)) !== null) {
    const fontPath = fontMatch[1];
    const fullFontPath = path.join(distDir, 'assets', fontPath);
    
    if (fs.existsSync(fullFontPath)) {
        try {
            const fontContent = fs.readFileSync(fullFontPath);
            const ext = path.extname(fontPath).toLowerCase();
            const mimeType = {
                '.woff': 'font/woff',
                '.woff2': 'font/woff2',
                '.ttf': 'font/ttf',
                '.otf': 'font/otf',
                '.eot': 'application/vnd.ms-fontobject'
            }[ext] || 'font/woff2';
            
            const base64 = fontContent.toString('base64');
            const dataUri = `data:${mimeType};base64,${base64}`;
            html = html.replace(fontMatch[0], `url(${dataUri})`);
            fontCount++;
        } catch (e) {
            console.warn(`⚠️ Could not encode font ${fontPath}: ${e.message}`);
        }
    }
}
console.log(`✅ Inlined ${fontCount} fonts`);

// 写入输出文件
fs.writeFileSync(outputFile, html, 'utf-8');

const fileSizeKB = Math.round(fs.statSync(outputFile).size / 1024);
console.log(`\n🎉 Bundle created successfully!`);
console.log(`📄 Output: ${outputFile}`);
console.log(`📦 File size: ${fileSizeKB} KB`);
