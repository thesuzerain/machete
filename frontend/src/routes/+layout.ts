// This tells SvelteKit to handle routing on the client side
export const ssr = false;
export const prerender = false;

// Handle all routes through the client
export const trailingSlash = 'never';