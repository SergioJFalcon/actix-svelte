import type { ServerLoadEvent } from "@sveltejs/kit";

export const load = async({ fetch }: ServerLoadEvent) => {
  console.log("Loading data from server");
  const environment = import.meta.env.MODE;
  console.log("Environment: ", environment);
  
  try {
    const response = await fetch(`/api/state`);
    if (!response.ok) {
      throw new Error(`Failed to fetch /api/state: ${response.statusText}`);
    }
    const data = await response.json();

    return data;
  //   return {
  //     app_name: 'Actix Svelte Example',
  //     app_version: '0.0.0',
  //     app_description: '',
  // };
  } catch (error) {
    console.error("Error loading data:", error);
    return {
        app_name: 'Actix Svelte Example',
        app_version: '0.0.0',
        app_description: '',
    };
  }
};