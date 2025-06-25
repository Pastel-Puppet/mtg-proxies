import { defineConfig } from 'vite';
import htmlMinifier from 'vite-plugin-html-minifier';

export default defineConfig({
    root: "www",
    base: "https://mtg-proxies.pastel-puppet.workers.dev/",
    build: {
        target: "esnext",
        assetsInlineLimit: 10240, 
    },
    plugins: [
        htmlMinifier({
            minify: true,
        }),
    ],
})