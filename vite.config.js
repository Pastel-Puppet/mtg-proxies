import { defineConfig } from 'vite';

export default defineConfig({
    root: "www",
    base: "https://mtg-proxies.pastel-puppet.workers.dev/",
    build: {
        target: "esnext",
        assetsInlineLimit: 10240, 
    },
})