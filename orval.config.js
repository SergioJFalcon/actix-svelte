module.exports = {
  'client': {
    input: './openapi.json',
    output: {
      target: './client/src/lib/client.ts',
      // This is the magic part!
      client: 'svelte-query',
      override: {
        // Here we tell orval to use our custom axios instance
        mutator: {
          path: './src/lib/api/axios.ts', // Path to our mutator
          name: 'customInstance',       // Name of the exported function
        },
        // Specify the version of TanStack Query you're using. v5 is the latest.
        // If you are on v4 as requested, use '4'.
        query: {
            version: 5, // Or 4
        }
      },
    },
    hooks: {
      // Optional: Run a linter/formatter after generation
      afterAllFilesWrite: 'pnpx eslint --fix ./web/src/lib/client.ts',
    },
  },
};