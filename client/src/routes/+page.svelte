<script lang="ts">
  import { onMount } from "svelte";

  let { data, children }: { data: any; children?: any } = $props();
  let counter_data = $state(data.counter);
  const incrementCounter = async () => {
    const res = await fetch('/api/counter', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      }
    }).then(res => res.json()).then(updated_data => {
      counter_data = updated_data;
    });
    console.log(data);
  };
  onMount(() => {
    console.log("Component mounted");
    console.log("Data:", data);
  });
</script>

<main class="">
  <div class="">
    <p>Counter: {counter_data}</p>
    <button onclick={incrementCounter} class="">Increment Counter</button>
    <p>Data: {JSON.stringify(data)}</p>
  </div>
</main>