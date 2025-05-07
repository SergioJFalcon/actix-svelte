import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';
import tailwindcss from '@tailwindcss/vite';

/** @type {import('vite').UserConfig} */
export default ({ mode }: { mode: string }) => {
  const env = loadEnv(mode, process.cwd());
  console.log("Vite env: ", env);
  console.log("Vite env mode: ", mode);
  console.log("Vite env PORT: ", env.VITE_PORT);
	let envSettings = {};
	if (mode === 'development') {
		envSettings = {
			server: {
				port: 3000,
				proxy: {
					'/api': `http://127.0.0.1:${env.VITE_PORT}`,
				}
			},
		};
	} else {
    envSettings = {
      server: {
        port: 3000,
        proxy: {
          '/api': `http://127.0.0.1:${env.VITE_PORT}`,
        }
      }
    }
  }
	return defineConfig({
		plugins: [
      sveltekit(),
      tailwindcss()
    ],
		...envSettings,
	});
};