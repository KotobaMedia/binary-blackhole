import React from "react";
import ChatBox from "./ChatMapPage/ChatBox";
import MainMap from "./ChatMapPage/MainMap";
import LayerSelector from "./ChatMapPage/LayerSelector";
import FeatureDetailsPanel from "./ChatMapPage/FeatureDetailsPanel";

const ChatMapPage: React.FC = () => {
  return (
    <div className="container-fluid vh-100">
      <div className="row h-100">
        <ChatBox />
        <div className="col-5 p-0">
          <div className="d-flex flex-column h-100">
            <div className="d-flex flex-wrap px-2 py-1 bg-danger text-center align-items-center justify-content-center">
              <span>
                <strong>
                  EXPERIMENTAL ・ 実験中 ・ すべてのクエリはログされています ・ <a href="https://github.com/KotobaMedia/binary-blackhole" target="_blank" rel="noopener noreferrer">GitHub はこちら</a> ・ 実験用データは一部<a href="https://nlftp.mlit.go.jp/ksj/" target="_blank" rel="noopener noreferrer">国土数値情報</a>を使用しています
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
    </div>
  );
}

export default ChatMapPage;
