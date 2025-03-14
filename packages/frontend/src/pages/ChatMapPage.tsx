import React from "react";
import c from "classnames";
import ChatBox from "./ChatMapPage/ChatBox";
import MainMap from "./ChatMapPage/MainMap";
import LayerSelector from "./ChatMapPage/LayerSelector";
import { useAtomValue } from "jotai";
import {
  detailPaneFullscreenAtom,
  detailPaneVisibleAtom,
} from "./ChatMapPage/atoms";
// import FeatureDetailsPanel from "./ChatMapPage/FeatureDetailsPanel";
import "./ChatMapPage/style.scss";
import FeatureDetailsPanel from "./ChatMapPage/FeatureDetailsPanel";

const ChatMapPage: React.FC = () => {
  const detailPaneVisible = useAtomValue(detailPaneVisibleAtom);
  const detailPaneFullscreen = useAtomValue(detailPaneFullscreenAtom);

  return (
    <div className="container-fluid overflow-hidden">
      <div
        className={c("row vh-100 vw-100 flex-nowrap chat-map-page-container", {
          "slide-left": detailPaneVisible,
          fullscreen: detailPaneFullscreen,
        })}
      >
        <div className={c("col-6 p-0 h-100")}>
          <ChatBox />
        </div>
        <div className="col-6 p-0">
          <div className="d-flex flex-column h-100">
            <div className="d-flex flex-wrap px-2 py-1 bg-danger text-center align-items-center justify-content-center">
              <span>
                <strong>
                  EXPERIMENTAL ・ 実験中 ・ すべてのクエリはログされています ・{" "}
                  <a
                    href="https://github.com/KotobaMedia/binary-blackhole"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    GitHub はこちら
                  </a>{" "}
                  ・ 実験用データは一部
                  <a
                    href="https://nlftp.mlit.go.jp/ksj/"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    国土数値情報
                  </a>
                  を使用しています
                </strong>
              </span>
            </div>
            <div className="flex-grow-1 position-relative">
              <MainMap />
            </div>
            <LayerSelector />
          </div>
        </div>
        <div
          className={c("p-0 detail-pane", {
            "col-6": !detailPaneFullscreen,
            "col-12": detailPaneFullscreen,
          })}
        >
          <FeatureDetailsPanel />
        </div>
      </div>
    </div>
  );
};

export default ChatMapPage;
