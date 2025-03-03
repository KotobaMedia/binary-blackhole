import React, { useCallback, useEffect, useRef, useState } from "react";
import Maplibre, { Source, Layer, MapRef } from 'react-map-gl/maplibre';
import { layersAtom, SQLLayer } from "./atoms";
import { useAtomValue } from "jotai";
import useSWR from 'swr';
import chroma from 'chroma-js';

type QueryResponse = {
  data: GeoJSON.FeatureCollection;
  bbox?: BBox;
}

// Type for bounding box
type BBox = [number, number, number, number]; // [west, south, east, north]

const queryFetcher = async (sql: string) => {
  const apiUrl = import.meta.env.VITE_API_URL;
  const response = await fetch(`${apiUrl}/query`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ query: sql }),
  });

  if (!response.ok) {
    throw new Error(`HTTP error! Status: ${response.status}`);
  }

  const result = await response.json();
  return result as QueryResponse;
};

// Function to generate a deterministic color based on layer name
const getLayerColor = (layerName: string) => {
  // Generate a hash from the string to get a deterministic number
  let hash = 0;
  for (let i = 0; i < layerName.length; i++) {
    hash = layerName.charCodeAt(i) + ((hash << 5) - hash);
  }

  // Use the hash to create a deterministic hue value
  const hue = hash % 360;

  // Create a vibrant, saturated color with chroma.js
  return chroma.hsl(hue, 0.7, 0.5).hex();
};

// Function to merge multiple bounding boxes
const mergeBboxes = (bboxes: BBox[]): BBox | null => {
  if (bboxes.length === 0) return null;

  return bboxes.reduce((merged, current) => {
    if (!merged) return current;
    return [
      Math.min(merged[0], current[0]), // min west
      Math.min(merged[1], current[1]), // min south
      Math.max(merged[2], current[2]), // max east
      Math.max(merged[3], current[3]), // max north
    ] as BBox;
  }, null as BBox | null);
};

const MapLayer: React.FC<{layer: SQLLayer, onBboxChange: (name: string, bbox: BBox | undefined) => void}> = ({layer, onBboxChange}) => {
  const { data: resp, error } = useSWR(
    layer.sql ? layer.sql : null,
    queryFetcher,
    { revalidateOnFocus: false }
  );

  useEffect(() => {
    // Update the parent component with this layer's bbox when it changes
    onBboxChange(layer.name, resp?.bbox);
  }, [resp?.bbox, layer.name, onBboxChange]);

  if (error) {
    console.error(`Error loading layer ${layer.name}:`, error);
    return null;
  }
  if (!resp) return null;

  const sourceId = `source-${layer.name}`;
  const layerColor = getLayerColor(layer.name);

  return (
    <Source id={sourceId} type="geojson" data={resp.data}>
      {/* Point layer */}
      <Layer
        id={`${layer.name}/point`}
        source={sourceId}
        type="circle"
        filter={['==', ['geometry-type'], 'Point']}
        paint={{
          'circle-radius': 5,
          'circle-color': layerColor,
          'circle-opacity': 0.8,
          'circle-stroke-width': 1,
          'circle-stroke-color': '#fff'
        }}
      />

      {/* Line layer */}
      <Layer
        id={`${layer.name}/line`}
        source={sourceId}
        type="line"
        filter={['==', ['geometry-type'], 'LineString']}
        paint={{
          'line-color': layerColor,
          'line-width': 2,
          'line-opacity': 0.8
        }}
      />

      {/* Polygon layer */}
      <Layer
        id={`${layer.name}/polygon-fill`}
        source={sourceId}
        type="fill"
        filter={['==', ['geometry-type'], 'Polygon']}
        paint={{
          'fill-color': layerColor,
          'fill-opacity': 0.4
        }}
      />

      {/* Polygon outline */}
      <Layer
        id={`${layer.name}/polygon-outline`}
        source={sourceId}
        type="line"
        filter={['==', ['geometry-type'], 'Polygon']}
        paint={{
          'line-color': layerColor,
          'line-width': 1,
          'line-opacity': 0.8
        }}
      />
    </Source>
  );
};

const MainMap: React.FC = () => {
  const layers = useAtomValue(layersAtom).filter(layer => layer.enabled);
  const [layerBboxes, setLayerBboxes] = useState<Record<string, BBox | undefined>>({});
  const mapRef = useRef<MapRef>(null);

  // Handle bbox updates from individual layers
  const handleBboxChange = useCallback((layerName: string, bbox: BBox | undefined) => {
    setLayerBboxes(prev => ({
      ...prev,
      [layerName]: bbox
    }));
  }, []);

  // Calculate merged bbox and fit map when bboxes change
  useEffect(() => {
    const bboxes = Object.values(layerBboxes).filter(
      (bbox): bbox is BBox => bbox !== undefined
    );

    if (bboxes.length > 0 && mapRef.current) {
      const mergedBbox = mergeBboxes(bboxes);
      if (mergedBbox) {
        // Add padding to the bbox
        mapRef.current.fitBounds(
          [[mergedBbox[0], mergedBbox[1]], [mergedBbox[2], mergedBbox[3]]],
          { padding: 50, duration: 1000 }
        );
      }
    }
  }, [layerBboxes]);

  return (
    <Maplibre
      ref={mapRef}
      mapStyle={"https://demotiles.maplibre.org/style.json"}
      initialViewState={{
        longitude: 135,
        latitude: 37,
        zoom: 4.0,
      }}
    >
      {layers.map(layer => (
        <MapLayer
          key={layer.name}
          layer={layer}
          onBboxChange={handleBboxChange}
        />
      ))}
    </Maplibre>
  );
}

export default MainMap;
