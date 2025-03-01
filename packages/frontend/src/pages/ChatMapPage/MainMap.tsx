import React from "react";
import Maplibre from 'react-map-gl/maplibre';

const MainMap: React.FC = () => {
  return (
    <Maplibre
      mapStyle={"https://demotiles.maplibre.org/style.json"}
      // hash={true}
      initialViewState={{
        longitude: 135,
        latitude: 37,
        zoom: 4.0,
      }}
    />
  );
}

export default MainMap;
