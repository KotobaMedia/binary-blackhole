import React from "react";
import { useLocation } from "wouter";
import AppLayout from "../components/AppLayout";
import useSWR from "swr";
import { fetcher } from "../tools/api";
import { createThread } from "../tools/threads";

const ChatListPage: React.FC = () => {
  const [, navigate] = useLocation();
  const { data } = useSWR<{ threads: { id: string; title: string }[] }>(
    "/threads",
    {
      fetcher,
    },
  );

  const handleNewChat = async () => {
    try {
      const threadId = await createThread();
      navigate(`/chats/${threadId}`);
    } catch (error) {
      console.error("Failed to create new chat:", error);
    }
  };

  return (
    <AppLayout>
      <div className="d-flex justify-content-between align-items-center mb-3">
        <h1>チャット履歴</h1>
        <button onClick={handleNewChat} className="btn btn-primary">
          新しいチャット
        </button>
      </div>
      <ul className="list-group">
        {data?.threads.map((thread) => (
          <li key={thread.id} className="list-group-item">
            <a href={`/chats/${thread.id}`} className="text-decoration-none">
              {thread.title}
            </a>
          </li>
        ))}
      </ul>
    </AppLayout>
  );
};

export default ChatListPage;
