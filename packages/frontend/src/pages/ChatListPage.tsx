import React from "react";
import AppLayout from "../components/AppLayout";
import useSWR from "swr";
import { fetcher } from "../tools/api";

const ChatListPage: React.FC = () => {
  const { data } = useSWR<{ threads: { id: string; title: string }[] }>(
    "/threads",
    {
      fetcher,
    },
  );
  return (
    <AppLayout>
      <ul>
        {data?.threads.map((thread) => (
          <li key={thread.id}>
            <a href={`/chats/${thread.id}`}>{thread.title}</a>
          </li>
        ))}
      </ul>
    </AppLayout>
  );
};

export default ChatListPage;
