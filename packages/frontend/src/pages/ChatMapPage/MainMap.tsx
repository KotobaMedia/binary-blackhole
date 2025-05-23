import React, { useCallback, useEffect, useRef, useState } from "react";
import Maplibre, {
  Source,
  Layer,
  MapRef,
  MapLayerMouseEvent,
} from "react-map-gl/maplibre";
import {
  layersAtom,
  SQLLayer,
  selectedFeaturesAtom,
  SelectedFeatureInfo,
  detailPaneVisibleAtom,
  mergedLayersAtom,
} from "./atoms";
import { useAtom, useAtomValue, useSetAtom } from "jotai";
import chroma from "chroma-js";
import { BBox, useQueryMetadata } from "../../tools/query";

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

const getLayerSourceId = (layer: SQLLayer) => `overlay-${layer.id}`;
const getStyleLayerId = (layer: SQLLayer, type: string) =>
  `overlay/${layer.id}/${type}`;

// Function to merge multiple bounding boxes
const mergeBboxes = (bboxes: BBox[]): BBox | undefined => {
  if (bboxes.length === 0) return undefined;

  return bboxes.reduce(
    (merged, current) => {
      if (!merged) return current;
      return [
        Math.min(merged[0], current[0]), // min west
        Math.min(merged[1], current[1]), // min south
        Math.max(merged[2], current[2]), // max east
        Math.max(merged[3], current[3]), // max north
      ] as BBox;
    },
    undefined as BBox | undefined,
  );
};

const MapLayer: React.FC<{
  layer: SQLLayer;
  onBboxChange: (name: string, bbox: BBox | undefined) => void;
}> = ({ layer, onBboxChange }) => {
  const { data: resp, error } = useQueryMetadata(layer.id);
  const setLayers = useSetAtom(layersAtom);

  useEffect(() => {
    // Update the parent component with this layer's bbox when it changes
    onBboxChange(layer.name, resp?.bounds);
  }, [resp?.bounds, layer.name, onBboxChange]);

  useEffect(() => {
    if (error) {
      setLayers((prev) =>
        prev.map((l) =>
          l.name === layer.name ? { ...l, error: error.message } : l,
        ),
      );
    }
  }, [error, layer.name, setLayers]);

  if (error) {
    console.error(`Error loading layer ${layer.name}:`, error);
    return <></>;
  }
  if (!resp) return <></>;

  const sourceId = getLayerSourceId(layer);
  const layerColor = getLayerColor(layer.name);

  return (
    <Source id={sourceId} type="vector" {...resp}>
      {/* Point layer */}
      <Layer
        id={getStyleLayerId(layer, "point")}
        source={sourceId}
        source-layer="data"
        type="circle"
        filter={["==", ["geometry-type"], "Point"]}
        paint={{
          "circle-radius": 5,
          "circle-color": layerColor,
          "circle-opacity": 0.8,
          "circle-stroke-width": 1,
          "circle-stroke-color": "#fff",
        }}
      />

      {/* Line layer */}
      <Layer
        id={getStyleLayerId(layer, "line")}
        source={sourceId}
        source-layer="data"
        type="line"
        filter={["==", ["geometry-type"], "LineString"]}
        paint={{
          "line-color": layerColor,
          "line-width": 2,
          "line-opacity": 0.8,
        }}
      />

      {/* Polygon layer */}
      <Layer
        id={getStyleLayerId(layer, "polygon-fill")}
        source={sourceId}
        source-layer="data"
        type="fill"
        filter={["==", ["geometry-type"], "Polygon"]}
        paint={{
          "fill-color": layerColor,
          "fill-opacity": 0.4,
        }}
      />

      {/* Polygon outline */}
      <Layer
        id={getStyleLayerId(layer, "polygon-outline")}
        source={sourceId}
        source-layer="data"
        type="line"
        filter={["==", ["geometry-type"], "Polygon"]}
        paint={{
          "line-color": layerColor,
          "line-width": 1,
          "line-opacity": 0.8,
        }}
      />

      {/* Selected outline for Points */}
      <Layer
        id={getStyleLayerId(layer, "point-selected-outline")}
        source={sourceId}
        source-layer="data"
        type="circle"
        filter={["==", ["geometry-type"], "Point"]}
        paint={{
          "circle-radius": 7, // slightly larger than the data circle to create a gap
          "circle-color": "transparent",
          "circle-stroke-color": "#333333", // dark grey outline
          "circle-stroke-width": 2,
          "circle-stroke-opacity": [
            "case",
            ["boolean", ["feature-state", "selected"], false],
            1,
            0,
          ],
        }}
      />

      {/* Selected outline for Lines */}
      <Layer
        id={getStyleLayerId(layer, "line-selected-outline")}
        source={sourceId}
        source-layer="data"
        type="line"
        filter={["==", ["geometry-type"], "LineString"]}
        paint={{
          "line-color": "#333333", // dark grey outline
          "line-width": [
            "case",
            ["boolean", ["feature-state", "selected"], false],
            4,
            0,
          ],
          "line-opacity": 1,
        }}
      />

      {/* Selected outline for Polygons */}
      <Layer
        id={getStyleLayerId(layer, "polygon-selected-outline")}
        source={sourceId}
        source-layer="data"
        type="line"
        filter={["==", ["geometry-type"], "Polygon"]}
        paint={{
          "line-color": "#333333", // dark grey outline
          "line-width": [
            "case",
            ["boolean", ["feature-state", "selected"], false],
            3,
            0,
          ],
          "line-opacity": 1,
        }}
      />
    </Source>
  );
};

const MainMap: React.FC = () => {
  const layers = useAtomValue(mergedLayersAtom).filter(
    (layer) => layer.enabled && !layer.error,
  );
  const [layerBboxes, setLayerBboxes] = useState<
    Record<string, BBox | undefined>
  >({});
  const mapRef = useRef<MapRef>(null);
  const [selectedFeatures, setSelectedFeatures] = useAtom(selectedFeaturesAtom);
  const setDetailPaneVisible = useSetAtom(detailPaneVisibleAtom);

  // Handle bbox updates from individual layers
  const handleBboxChange = useCallback(
    (layerName: string, bbox: BBox | undefined) => {
      setLayerBboxes((prev) => ({
        ...prev,
        [layerName]: bbox,
      }));
    },
    [],
  );

  // Handle click on map features
  const handleMapClick = useCallback(
    (event: MapLayerMouseEvent) => {
      if (!mapRef.current) return;

      const map = mapRef.current.getMap();
      // Get all visible layers that we've added
      const visibleLayers = layers
        .map((layer) => [
          getStyleLayerId(layer, "point"),
          getStyleLayerId(layer, "line"),
          getStyleLayerId(layer, "polygon-fill"),
          getStyleLayerId(layer, "polygon-outline"),
        ])
        .flat();

      // Query features at the clicked point
      const features = map.queryRenderedFeatures(event.point, {
        layers: visibleLayers,
      });

      if (features.length > 0) {
        console.log("Clicked features:", features);

        // Format feature information and store in the atom
        const formattedFeatures: SelectedFeatureInfo[] = features
          .map((feature) => {
            const layerId = feature.layer.id;
            const sqlLayerId = layerId.split("/")[1];
            const geometryType = layerId.split("/")[2];

            const sqlLayer = layers.find((layer) => layer.id === sqlLayerId);
            if (!sqlLayer) {
              return null;
            }

            // Log for debugging
            console.log(
              `Feature from layer: ${sqlLayer?.name} (${geometryType})`,
            );
            console.log("Properties:", feature.properties);
            console.log("Geometry type:", feature.geometry.type);
            console.log("-------------------");

            return {
              feature,
              layer: sqlLayer,
              geometryType,
            };
          })
          .filter((item): item is SelectedFeatureInfo => item !== null);

        // Update the atom with selected features
        setSelectedFeatures(formattedFeatures);
        setDetailPaneVisible(true);
      } else {
        console.log("No features found at this location");
        // Clear selected features when clicking on empty space
        setSelectedFeatures([]);
      }
    },
    [layers, setDetailPaneVisible, setSelectedFeatures],
  );

  // Calculate merged bbox and fit map when bboxes change
  useEffect(() => {
    const bboxes = Object.values(layerBboxes).filter(
      (bbox): bbox is BBox => !!bbox,
    );

    if (bboxes.length > 0 && mapRef.current) {
      const mergedBbox = mergeBboxes(bboxes);
      if (mergedBbox) {
        // Add padding to the bbox
        mapRef.current.fitBounds(
          [
            [mergedBbox[0], mergedBbox[1]],
            [mergedBbox[2], mergedBbox[3]],
          ],
          { padding: 50, duration: 1000 },
        );
      }
    }
  }, [layerBboxes]);

  useEffect(() => {
    if (!mapRef.current) return;
    const map = mapRef.current.getMap();
    const f = selectedFeatures;
    for (const { layer, feature } of f) {
      const sourceId = getLayerSourceId(layer);
      const featureId = feature.id;
      if (featureId !== undefined) {
        map.setFeatureState(
          { source: sourceId, id: featureId, sourceLayer: "data" },
          { selected: true },
        );
      }
    }
    return () => {
      for (const { layer, feature } of f) {
        const sourceId = getLayerSourceId(layer);
        const featureId = feature.id;
        if (featureId !== undefined) {
          map.removeFeatureState({
            source: sourceId,
            id: featureId,
            sourceLayer: "data",
          });
        }
      }
    };
  }, [selectedFeatures]);

  return (
    <Maplibre
      ref={mapRef}
      mapStyle={"https://tiles.kmproj.com/styles/osm-en-white.json"}
      initialViewState={{
        longitude: 135,
        latitude: 37,
        zoom: 4.0,
      }}
      onClick={handleMapClick}
    >
      {layers.map((layer) => (
        <MapLayer
          key={layer.name}
          layer={layer}
          onBboxChange={handleBboxChange}
        />
      ))}
    </Maplibre>
  );
};

export default MainMap;
