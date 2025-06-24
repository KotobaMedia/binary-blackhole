import React from "react";
import { Redirect, useRoute } from "wouter";
import ChatMapPage from "./ChatMapPage";

const ChatMapPageWrapper: React.FC = () => {
  const [match, params] = useRoute("/chats/:threadId");

  if (!match || !params.threadId) {
    return <Redirect to="/" />;
  }

  return <ChatMapPage threadId={params.threadId} />;
};

export default ChatMapPageWrapper;
