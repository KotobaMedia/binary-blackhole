import { useAtom } from "jotai";
import React, { useCallback } from "react";
import { layersAtom } from "./atoms";
import { ExclamationTriangleFill } from "react-bootstrap-icons";

const LayerSelector: React.FC = () => {
  const [layers, setLayers] = useAtom(layersAtom);

  const toggleLayer = useCallback((layerName: string) => {
    setLayers(prev => prev.map(layer => {
      if (layer.name === layerName) {
        return { ...layer, enabled: !layer.enabled };
      }
      return layer;
    }));
  }, [setLayers]);

  return (
    <div className="d-flex flex-wrap gap-2 px-2">
      {layers.map((layer) => (
        <button
          key={layer.name}
          className={`btn btn-sm my-2 ${layer.enabled ? 'btn-primary' : 'btn-outline-secondary'}`}
          onClick={() => toggleLayer(layer.name)}
          type="button"
        >
          {layer.error ? <ExclamationTriangleFill /> : null}
          {layer.name}
        </button>
      ))}
    </div>
  );
}

export default LayerSelector;
