import adapter from '@sveltejs/adapter-auto';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		adapter: adapter(),
		paths: {
			base: ''
		},
		prerender: {
			handleHttpError: ({ path, referrer, message }) => {
				// Ignore 404s on prerendering
				if (message.includes('Not found')) return;
				throw new Error(message);
			}
		}
	},
	preprocess: vitePreprocess()
};

export default config;
