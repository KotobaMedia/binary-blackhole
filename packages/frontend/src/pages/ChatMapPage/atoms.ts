import { atom } from "jotai";
import type { MapGeoJSONFeature } from "react-map-gl/maplibre";

export type SQLLayer = {
  name: string;
  sql: string;
  enabled: boolean;
  error?: string;
};

export type SelectedFeatureInfo = {
  feature: MapGeoJSONFeature;
  layerName: string;
  geometryType: string;
};

export const layersAtom = atom<SQLLayer[]>([]);
export const selectedFeaturesAtom = atom<SelectedFeatureInfo[]>([]);

export const detailPaneVisibleAtom = atom(false);
export const detailPaneFullscreenAtom = atom(false);
