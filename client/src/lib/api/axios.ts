import Axios, { type AxiosRequestConfig } from 'axios';

// Use the environment variable for the backend URL, with a fallback for local dev
export const API_URL = import.meta.env.VITE_API_URL || 'http://127.0.0.1:8080';

export const AXIOS_INSTANCE = Axios.create({ baseURL: API_URL });

// This is the custom instance that orval will use
// It takes a generic type T for the response data
export const customInstance = <T>(
  config: AxiosRequestConfig,
): Promise<T> => {
  return AXIOS_INSTANCE(config).then(({ data }) => data);
};

// You can export other things here for different use cases if needed
// For example, a version that returns the full AxiosResponse
// export const customInstanceWithFullResponse = <T>( ... );