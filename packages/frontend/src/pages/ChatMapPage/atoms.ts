import { atom } from "jotai";
import type { MapGeoJSONFeature } from "react-map-gl/maplibre";

export type SQLLayer = {
  id: string;
  name: string;
  sql: string;
  enabled: boolean;
  error?: string;
};

export type SelectedFeatureInfo = {
  feature: MapGeoJSONFeature;
  layer: SQLLayer;
  geometryType: string;
};

/// All layers, including previous versions.
export const layersAtom = atom<SQLLayer[]>([]);

/// This is the list of layers with previous versions removed.
export const mergedLayersAtom = atom<SQLLayer[]>((get) => {
  const allLayers = get(layersAtom);
  const dedupedLayers = allLayers.reduce((acc: SQLLayer[], layer: SQLLayer) => {
    const existingLayerIdx = acc.findIndex((l) => l.id === layer.id);
    if (existingLayerIdx >= 0) {
      // replace the existing layer with the new one
      acc.splice(existingLayerIdx, 1, layer);
    } else {
      acc.push(layer);
    }
    return acc;
  }, []);
  return dedupedLayers;
});
export const enabledLayersAtom = atom<SQLLayer[]>((get) => {
  const allLayers = get(mergedLayersAtom);
  const enabledLayers = allLayers.filter((layer) => layer.enabled);
  return enabledLayers;
});

export const selectedFeaturesAtom = atom<SelectedFeatureInfo[]>([]);

export const detailPaneVisibleAtom = atom(false);
