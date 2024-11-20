import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';

const env = loadEnv(process.env.NODE_ENV as string, process.cwd());

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		proxy: {
			'/api': {
				target: env.PUBLIC_API_URL ?? 'http://localhost:8000',
				changeOrigin: true,
				rewrite: (path) => path.replace(/^\/api/, '')
			}
		},
		fs: {
			allow: ['..']
		}
	}
});

