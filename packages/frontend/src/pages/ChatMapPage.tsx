import React from "react";
import ChatBox from "./ChatMapPage/ChatBox";
import MainMap from "./ChatMapPage/MainMap";
import LayerSelector from "./ChatMapPage/LayerSelector";
import { useAtomValue, useSetAtom } from "jotai";
import {
  detailPaneVisibleAtom,
  mobileMapVisibleAtom,
} from "./ChatMapPage/atoms";
import { Map, ChatDots } from "react-bootstrap-icons";
import clsx from "clsx";
import "./ChatMapPage/style.scss";
import FeatureDetailsPanel from "./ChatMapPage/FeatureDetailsPanel";

type ChatMapPageProps = {
  threadId: string;
};

const ExperimentalBanner: React.FC = () => (
  <div className="d-flex flex-wrap px-2 py-1 bg-danger-subtle text-center align-items-center justify-content-center">
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
);

const ChatMapPage: React.FC<ChatMapPageProps> = ({ threadId }) => {
  const detailPaneVisible = useAtomValue(detailPaneVisibleAtom);
  const mobileMapVisible = useAtomValue(mobileMapVisibleAtom);
  const setMobileMapVisible = useSetAtom(mobileMapVisibleAtom);

  const toggleMobileMap = () => {
    setMobileMapVisible(!mobileMapVisible);
  };

  return (
    <>
      {/* Mobile Layout */}
      <div className="d-md-none vh-100 w-100 d-flex flex-column">
        {/* Mobile Header with Toggle Button */}
        <div className="d-flex justify-content-between align-items-center p-2 bg-light border-bottom">
          <span className="fw-bold">チャット</span>
          <button
            className="btn btn-outline-primary btn-sm"
            onClick={toggleMobileMap}
          >
            {mobileMapVisible ? (
              <>
                <ChatDots className="me-1" />
                チャット
              </>
            ) : (
              <>
                <Map className="me-1" />
                マップ
              </>
            )}
          </button>
        </div>

        {/* Mobile Content - Always render both, show/hide with CSS */}
        <div className="flex-grow-1 position-relative">
          {/* Chat View */}
          <div
            className={clsx("h-100", {
              "d-none": mobileMapVisible,
              "d-block": !mobileMapVisible,
            })}
          >
            <ChatBox threadId={threadId} />
          </div>

          {/* Map View */}
          <div
            className={clsx("h-100", "d-flex", "flex-column", {
              "d-flex": mobileMapVisible,
              "d-none": !mobileMapVisible,
            })}
          >
            <ExperimentalBanner />
            <div className="flex-grow-1 position-relative">
              <MainMap />
            </div>
            <LayerSelector />
            {detailPaneVisible && (
              <div className="detail-pane">
                <FeatureDetailsPanel />
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Desktop Layout */}
      <div className="d-none d-md-block">
        <div className="container-fluid overflow-hidden">
          <div className="row vh-100 vw-100 flex-nowrap chat-map-page-container">
            <div className="col-4 p-0 h-100">
              <ChatBox threadId={threadId} />
            </div>
            <div className="col-8 p-0">
              <div className="d-flex flex-column h-100">
                <ExperimentalBanner />
                <div className="flex-grow-1 position-relative">
                  <MainMap />
                </div>
                <LayerSelector />
                {detailPaneVisible && (
                  <div className="detail-pane">
                    <FeatureDetailsPanel />
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
};

export default ChatMapPage;
