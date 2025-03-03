import React from "react";
import ChatBox from "./ChatMapPage/ChatBox";
import MainMap from "./ChatMapPage/MainMap";
import LayerSelector from "./ChatMapPage/LayerSelector";
import FeatureDetailsPanel from "./ChatMapPage/FeatureDetailsPanel";

const ChatMapPage: React.FC = () => {
  return (
    <div className="row bg-body text-body h-100">
      <ChatBox />
      <div className="col-5 p-0">
        <div className="d-flex flex-column h-100">
          <div className="d-flex flex-wrap px-2 bg-danger align-items-center justify-content-center">
            <span>
              <strong>
                EXPERIMENTAL - 実験用 - <a href="https://github.com/KotobaMedia/binary-blackhole" target="_blank" rel="noopener noreferrer">GitHub で連絡</a>
              </strong>
            </span>
          </div>
          <div className="flex-grow-1 position-relative">
            <MainMap />
          </div>
          <LayerSelector />
        </div>
      </div>
      <div className="col-3 p-0 h-100 overflow-y-auto overflow-x-hidden">
        <FeatureDetailsPanel />
      </div>
    </div>
  );
}

export default ChatMapPage;
