///! Hooks, types, and functions to query the database

import useSWR, { BareFetcher, SWRConfiguration } from "swr";

export type QueryResponse = {
  data: GeoJSON.FeatureCollection;
  bbox?: BBox;
};

// Type for bounding box
export type BBox = [number, number, number, number]; // [west, south, east, north]

const queryFetcher = async (sql: string) => {
  const apiUrl = import.meta.env.VITE_API_URL;
  const response = await fetch(`${apiUrl}/query`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ query: sql }),
  });

  if (!response.ok) {
    throw new Error(`HTTP error! Status: ${response.status}`);
  }

  const result = await response.json();
  return result as QueryResponse;
};

export const useQuery = (
  sql: string | undefined,
  config?: SWRConfiguration<QueryResponse, Error, BareFetcher<QueryResponse>>,
) => {
  return useSWR(!!sql ? sql : null, queryFetcher, {
    revalidateOnFocus: false,
    ...config,
  });
};
