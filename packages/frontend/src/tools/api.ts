// SWR fetcher function
export const fetcher = async (url: string) => {
  const apiUrl = import.meta.env.VITE_API_URL;
  if (!apiUrl) {
    throw new Error('API URL is not defined');
  }
  const res = await fetch(`${apiUrl}${url}`);
  if (!res.ok) {
    throw new Error('API request failed');
  }
  return res.json();
};
