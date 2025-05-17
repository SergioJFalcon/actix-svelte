<script lang="ts">
	import "../app.css";
	import { ModeWatcher } from 'mode-watcher';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	let { data, children }: { data: any; children?: any } = $props();
	let isLoading = $state(true);

	// Fetch data on mount
	onMount(async () => {
		try {
			const stateRes = await fetch('/api/state');
			let state_data = await stateRes.json();
			console.log("Layout State data:", state_data);

			data = state_data;
		} catch (error) {
			console.error("Error fetching data:", error);
		} finally {
			isLoading = false;
		}
	});
</script>

<main class="min-h-dvh w-dvw max-h-dvh overflow-hidden bg-slate-500 text-white">
	<header class="min-h-[10dvh]">
		<nav class="h-full w-full flex flex-row justify-between">
			<div class="h-full w-[10%]"></div>
			<div class="h-full w-[80%]">
				{#if isLoading}
					<h3>Not finished loading</h3>
				{:else}
					<h2 class="text-6xl font-bold">{data.app_name} - {data.app_version}</h2>
				{/if}
			</div>
		</nav>
	</header>
	<section class="h-[90dvh] w-full grid grid-cols-1 gap-4 px-8 py-8">
			{@render children()}
	</section>
</main>
