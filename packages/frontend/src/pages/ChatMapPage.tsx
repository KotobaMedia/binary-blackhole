import React from "react";
import ChatBox from "./ChatMapPage/ChatBox";
import MainMap from "./ChatMapPage/MainMap";
import LayerSelector from "./ChatMapPage/LayerSelector";

const ChatMapPage: React.FC = () => {
  return (
    <div className="row bg-body text-body h-100">
      <ChatBox />
      <div className="col-8 p-0">
        <div className="d-flex flex-column h-100">
          <MainMap />
          <LayerSelector />
        </div>
      </div>
    </div>
  );
}

export default ChatMapPage;
