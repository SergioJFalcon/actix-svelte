<script lang="ts">

  let { data, children }: { data: any; children?: any } = $props();
</script>

<main class="min-h-dvh w-dvw max-h-dvh overflow-hidden bg-slate-500 text-white">
  <header class="min-h-[10dvh]">
    <nav class="h-full w-full flex flex-row justify-between">
      <div class="h-full w-[10%]"></div>
      <div class="h-full w-[80%] text-center">
        <h2 class="text-6xl font-bold">{data.app_name}</h2>
      </div>
    </nav>
  </header>
  <div class="h-[90dvh]">
      {@render children()}
  </div>
</main>
