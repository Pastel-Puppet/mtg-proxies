import terser from '@rollup/plugin-terser';
import copy from 'rollup-plugin-copy';

export default {
    input: 'assets/interface.js',
    output: {
        file: 'dist/bundle.js',
        format: 'es',
        plugins: [terser()]
    },
    plugins: [
        copy({
            targets: [
                { src: ['assets/*.html', 'assets/*.css', 'assets/*.ico', 'assets/pkg/*.wasm'], dest: 'dist' }
            ]
        })
    ]
};