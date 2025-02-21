import React from "react";
import ChatBox from "../components/ChatBox";
import MainMap from "../components/MainMap";

const MainPage: React.FC = () => {
  return (
    <div className="row h-100 bg-dark text-light">
      <ChatBox />
      <MainMap />
    </div>
  );
}

export default MainPage;
