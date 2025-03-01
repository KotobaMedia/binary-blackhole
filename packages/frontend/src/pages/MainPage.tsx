import React from "react";
import ChatBox from "../components/ChatBox";
import MainMap from "../components/MainMap";

const MainPage: React.FC = () => {
  return (
    <div className="row bg-primary-subtle h-100">
      <ChatBox />
      <MainMap />
    </div>
  );
}

export default MainPage;
