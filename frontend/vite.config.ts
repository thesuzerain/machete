import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';

const env = loadEnv(process.env.NODE_ENV as string, process.cwd());
const API_URL = env.API_URL as string;

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		proxy: {
			[API_URL]: {
				target: 'http://localhost:8000',
				changeOrigin: true,
				rewrite: (path) => path.replace(/^\/api/, '')
			}
		},
		fs: {
			allow: ['..']
		}
	}
});

