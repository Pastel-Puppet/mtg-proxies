import terser from '@rollup/plugin-terser';
import copy from 'rollup-plugin-copy';

export default {
    input: 'wasm_proxies/interface.js',
    output: {
        file: 'www/bundle.js',
        format: 'es',
        plugins: [terser()]
    },
    plugins: [
        copy({
            targets: [
                { src: ['wasm_proxies/pkg/*.wasm'], dest: 'www' }
            ]
        })
    ]
};
