import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';


/** @type {import('vite').UserConfig} */
export default ({ mode }: { mode: string }) => {
	let envSettings = {};
	if (mode === 'development') {
		envSettings = {
			server: {
				port: 3000,
				proxy: {
					'/api': 'http://localhost:5000'
				}
			},
		};
	} else {
    envSettings = {
      server: {
        port: 3000,
        proxy: {
          '/api': 'http://localhost:5000'
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

// export default defineConfig({
// 	plugins: [
//     sveltekit(),
//     tailwindcss()
//   ]
// });
