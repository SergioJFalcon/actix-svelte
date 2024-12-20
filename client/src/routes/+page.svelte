<script lang="ts">
  let { data, children }: { data: any; children?: any } = $props();

  let count = $state(0);

  const counter = async () => {
    const res = await fetch('/api/counter', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      }
    }).then(res => res.json()).then(data => {
      count = data;
    });
    console.log(data);
  };
</script>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://svelte.dev/docs/kit">svelte.dev/docs/kit</a> to read the documentation</p>
<p>Counter: {count}</p>
<button onclick={counter}>Increment Counter</button>