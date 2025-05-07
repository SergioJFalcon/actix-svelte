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
  } catch (error) {
    console.error("Error loading data:", error);
    return {
        app_name: 'Actix Svelte',
        app_version: '0.0.0',
        app_description: '',
    };
  }
};

// export const load = async () => {
//   console.log("Loading data from server");
//   const environment = import.meta.env.MODE;
//   console.log("Environment: ", environment);
//   return {
//       app_name: 'Default Clean Room Dashboard',
//       app_version: '2.0.0',
//       app_description: 'A clean room dashboard',
//       counter: 0,
//       global_counter: 0
//   };
// };