import { useAtomValue, useSetAtom } from "jotai";
import React, { useCallback } from "react";
import { layersAtom, mergedLayersAtom } from "./atoms";
import { ExclamationTriangleFill } from "react-bootstrap-icons";
import clsx from "clsx";

const LayerSelector: React.FC = () => {
  const layers = useAtomValue(mergedLayersAtom);
  const setLayers = useSetAtom(layersAtom);

  const toggleLayer = useCallback(
    (layerName: string) => {
      setLayers((prev) =>
        prev.map((layer) => {
          if (layer.name === layerName) {
            return { ...layer, enabled: !layer.enabled };
          }
          return layer;
        }),
      );
    },
    [setLayers],
  );

  return (
    <div
      className={clsx("d-flex flex-wrap gap-2 p-2", {
        "d-none": !layers.length,
      })}
    >
      {layers.map((layer) => (
        <button
          key={layer.name}
          className={`btn btn-sm py-0 ${layer.enabled ? "btn-primary" : "btn-outline-secondary"}`}
          onClick={() => toggleLayer(layer.name)}
          type="button"
        >
          {layer.error ? <ExclamationTriangleFill /> : null}
          {layer.name}
        </button>
      ))}
    </div>
  );
};

export default LayerSelector;
