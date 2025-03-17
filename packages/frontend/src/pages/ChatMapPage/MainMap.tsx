import React, {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import Maplibre, {
  Source,
  Layer,
  MapRef,
  MapLayerMouseEvent,
  StyleSpecification,
} from "react-map-gl/maplibre";
import {
  layersAtom,
  SQLLayer,
  selectedFeaturesAtom,
  SelectedFeatureInfo,
  detailPaneVisibleAtom,
} from "./atoms";
import { useAtom, useAtomValue, useSetAtom } from "jotai";
import MainMapStyle from "./MainMapStyle.json";
import chroma from "chroma-js";
import { BBox, useQuery } from "../../tools/query";

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
  const { data: resp, error } = useQuery(layer.sql);
  const setLayers = useSetAtom(layersAtom);

  useEffect(() => {
    // Update the parent component with this layer's bbox when it changes
    onBboxChange(layer.name, resp?.bbox);
  }, [resp?.bbox, layer.name, onBboxChange]);

  useEffect(() => {
    if (error) {
      setLayers((prev) =>
        prev.map((l) =>
          l.name === layer.name ? { ...l, error: error.message } : l,
        ),
      );
    }
    if (resp?.data) {
      let count = resp?.data.features.length;
      if (count === 0) {
        setLayers((prev) =>
          prev.map((l) =>
            l.name === layer.name ? { ...l, error: "No features found" } : l,
          ),
        );
      }
    }
  }, [resp?.data, error, layer.name, setLayers]);

  const featureCollection = useMemo<GeoJSON.FeatureCollection | null>(() => {
    const data = resp?.data;
    if (!data) return null;
    return {
      type: "FeatureCollection",
      features: data.features.map((feature, idx) => ({
        id: feature.id ?? feature.properties?._id ?? idx,
        ...feature,
        properties: {
          ...feature.properties,
        },
      })),
    };
  }, [resp?.data]);

  if (error) {
    console.error(`Error loading layer ${layer.name}:`, error);
    return <></>;
  }
  if (!resp || !featureCollection) return <></>;
  if (featureCollection.features.length === 0) return <></>;

  const sourceId = `source-${layer.name}`;
  const layerColor = getLayerColor(layer.name);

  return (
    <Source id={sourceId} type="geojson" data={featureCollection}>
      {/* Point layer */}
      <Layer
        id={`${layer.name}/point`}
        source={sourceId}
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
        id={`${layer.name}/line`}
        source={sourceId}
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
        id={`${layer.name}/polygon-fill`}
        source={sourceId}
        type="fill"
        filter={["==", ["geometry-type"], "Polygon"]}
        paint={{
          "fill-color": layerColor,
          "fill-opacity": 0.4,
        }}
      />

      {/* Polygon outline */}
      <Layer
        id={`${layer.name}/polygon-outline`}
        source={sourceId}
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
        id={`${layer.name}/point-selected-outline`}
        source={sourceId}
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
        id={`${layer.name}/line-selected-outline`}
        source={sourceId}
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
        id={`${layer.name}/polygon-selected-outline`}
        source={sourceId}
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
  const layers = useAtomValue(layersAtom).filter(
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
          `${layer.name}/point`,
          `${layer.name}/line`,
          `${layer.name}/polygon-fill`,
          `${layer.name}/polygon-outline`,
        ])
        .flat();

      // Query features at the clicked point
      const features = map.queryRenderedFeatures(event.point, {
        layers: visibleLayers,
      });

      if (features.length > 0) {
        console.log("Clicked features:", features);

        // Format feature information and store in the atom
        const formattedFeatures: SelectedFeatureInfo[] = features.map(
          (feature) => {
            const layerId = feature.layer.id;
            const layerName = layerId.split("/")[0];
            const geometryType = layerId.split("/")[1];

            // Log for debugging
            console.log(`Feature from layer: ${layerName} (${geometryType})`);
            console.log("Properties:", feature.properties);
            console.log("Geometry type:", feature.geometry.type);
            console.log("-------------------");

            return {
              feature,
              layerName,
              geometryType,
            };
          },
        );

        // Update the atom with selected features
        setSelectedFeatures(formattedFeatures);
        setDetailPaneVisible(true);
      } else {
        console.log("No features found at this location");
        // Clear selected features when clicking on empty space
        setSelectedFeatures([]);
      }
    },
    [layers, setSelectedFeatures],
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
    for (const { layerName, feature } of f) {
      const sourceId = `source-${layerName}`;
      const featureId = feature.id;
      if (featureId !== undefined) {
        map.setFeatureState(
          { source: sourceId, id: featureId },
          { selected: true },
        );
      }
    }
    return () => {
      for (const { layerName, feature } of f) {
        const sourceId = `source-${layerName}`;
        const featureId = feature.id;
        if (featureId !== undefined) {
          map.removeFeatureState({ source: sourceId, id: featureId });
        }
      }
    };
  }, [selectedFeatures]);

  return (
    <Maplibre
      ref={mapRef}
      mapStyle={MainMapStyle as StyleSpecification}
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
