import { useAtomValue } from "jotai";
import React, { useState } from "react";
import { layersAtom } from "./atoms";

const LayerSelector: React.FC = () => {
  const layers = useAtomValue(layersAtom);
  const [activeLayers, setActiveLayers] = useState<{[key: string]: boolean}>({});

  const toggleLayer = (layerName: string) => {
    setActiveLayers(prev => ({
      ...prev,
      [layerName]: !prev[layerName]
    }));
  };

  return (
    <div className="d-flex flex-wrap gap-2">
      {layers.map((layer) => (
        <button
          key={layer.name}
          className={`btn btn-sm ${activeLayers[layer.name] ? 'btn-primary' : 'btn-outline-secondary'}`}
          onClick={() => toggleLayer(layer.name)}
          type="button"
        >
          {layer.name}
        </button>
      ))}
    </div>
  );
}

export default LayerSelector;
