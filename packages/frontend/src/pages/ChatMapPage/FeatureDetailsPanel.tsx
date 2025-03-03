import React, { useState } from "react";
import { useAtomValue } from "jotai";
import { SelectedFeatureInfo, selectedFeaturesAtom } from "./atoms";

// Feature Item component for individual feature display
const FeatureItem: React.FC<{ item: SelectedFeatureInfo; index: number }> = ({ item, index }) => (
  <div className="mb-2">
    <small className="text-muted d-block mb-1">{item.geometryType}</small>
    <div className="properties">
      <table className="table table-sm table-striped mb-0 small">
        <tbody>
          {Object.entries(item.feature.properties || {}).map(([key, value]) => (
            <tr key={key}>
              <td className="fw-bold px-1">{key}</td>
              <td className="px-1">{String(value)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  </div>
);

// Feature Group component for each layer
const FeatureGroup: React.FC<{ layerName: string; features: SelectedFeatureInfo[] }> = ({ layerName, features }) => {
  const [expanded, setExpanded] = useState(true);

  return (
    <div className="mb-3 p-1">
      <div
        className="d-flex justify-content-between align-items-center cursor-pointer"
        onClick={() => setExpanded(!expanded)}
        style={{ cursor: "pointer" }}
      >
        <h5 className="mb-0">{layerName} ({features.length})</h5>
        <span>{expanded ? "▼" : "◀︎"}</span>
      </div>

      {expanded && (
        <div className="mt-2">
          {features.map((item, index) => (
            <FeatureItem key={index} item={item} index={index} />
          ))}
        </div>
      )}
    </div>
  );
};

const FeatureDetailsPanel: React.FC = () => {
  const selectedFeatures = useAtomValue(selectedFeaturesAtom);

  if (selectedFeatures.length === 0) {
    return (
      <div className="feature-details-panel p-3 border-top">
        <p className="text-muted">地物は選択されていません。クエリー実行後に地物をクリックすると詳細をここで確認できます。</p>
      </div>
    );
  }

  // Group features by layerName
  const groupedFeatures = selectedFeatures.reduce((acc, feature) => {
    const { layerName } = feature;
    if (!acc[layerName]) {
      acc[layerName] = [];
    }
    acc[layerName].push(feature);
    return acc;
  }, {} as Record<string, typeof selectedFeatures>);

  return (
    <div className="feature-details-panel overflow-auto p-3">
      {Object.entries(groupedFeatures).map(([layerName, features]) => (
        <FeatureGroup key={layerName} layerName={layerName} features={features} />
      ))}
    </div>
  );
};

export default FeatureDetailsPanel;
