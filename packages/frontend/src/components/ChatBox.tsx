import React, { JSX, useCallback, useState, useRef, useEffect } from "react";
import ReactMarkdown from "react-markdown";
import logo from "/logo-small.svg?url";
import { QuestionCircleFill } from "react-bootstrap-icons";
import useSWR from "swr";

// Types for the API response data
type Role = "user" | "assistant" | "system" | "tool";

type ChatterMessageSidecar = "None" | {
  "SQLExecution": [string, string];
};

type ChatterMessageView = {
  message?: string;
  role: Role;
  sidecar?: ChatterMessageSidecar;
};

type Message = {
  id: number;
  content: ChatterMessageView;
};

type ThreadDetails = {
  id: string;
  title: string;
  messages: Message[];
};

type CreateThreadResponse = {
  thread_id: string;
};

// SWR fetcher function
const fetcher = (url: string) => fetch(url).then(res => {
  if (!res.ok) {
    throw new Error('API request failed');
  }
  return res.json();
});

const AssistantMessage: React.FC<React.PropsWithChildren> = ({children}) => {
  return (
    <div className="d-flex justify-content-start mb-2">
      <div className="p-2 rounded" style={{"maxWidth": "90%"}}>
        {children}
      </div>
    </div>
  );
}

const UserMessage: React.FC<React.PropsWithChildren> = ({children}) => {
  return (
    <div className="d-flex justify-content-end mb-2">
      <div className="bg-primary text-white p-2 rounded" style={{"maxWidth": "90%"}}>
        {children}
      </div>
    </div>
  );
}

type SendMessageBoxProps = {
  onSendMessage: (message: string) => void;
  isLoading: boolean;
};

const SendMessageBox: React.FC<SendMessageBoxProps> = ({ onSendMessage, isLoading }) => {
  const [message, setMessage] = useState('');

  const onSubmit = useCallback<React.FormEventHandler<HTMLFormElement>>((e) => {
    e.preventDefault();
    if (!message.trim() || isLoading) return;

    onSendMessage(message);
    setMessage('');
    // Reset the height of the textarea
    const textarea = e.currentTarget.querySelector('textarea');
    if (textarea) {
      textarea.style.height = 'auto';
    }
  }, [message, onSendMessage, isLoading]);

  const onInput = useCallback<React.FormEventHandler<HTMLTextAreaElement>>((e) => {
    const el = e.currentTarget;
    const maxHeight = 150; // maximum height in pixels
    el.style.height = "auto";
    if (el.scrollHeight < maxHeight) {
      el.style.height = `${el.scrollHeight}px`;
      el.style.overflowY = "hidden";
    } else {
      el.style.height = `${maxHeight}px`;
      el.style.overflowY = "auto";
    }
  }, []);

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setMessage(e.target.value);
  };

  return (
    <form onSubmit={onSubmit}>
      <div className="input-group">
        <textarea
          className="form-control"
          placeholder="何を調べましょうか？"
          rows={1}
          style={{ overflow: "hidden", resize: "none", maxHeight: "150px" }}
          onInput={onInput}
          value={message}
          onChange={handleChange}
          disabled={isLoading}
        />
        <button className="btn btn-primary" type="submit" disabled={isLoading}>
          {isLoading ? (
            <span className="spinner-border spinner-border-sm" role="status" aria-hidden="true"></span>
          ) : (
            <QuestionCircleFill className="align-baseline" title="Submit" />
          )}
        </button>
      </div>
    </form>
  );
}

const ChatBox: React.FC = () => {
  const [threadId, setThreadId] = useState<string | null>(null);
  const [isSending, setIsSending] = useState(false);
  const apiUrl = import.meta.env.VITE_API_URL;
  const messageContainerRef = useRef<HTMLDivElement>(null);

  const { data, error, isLoading, mutate } = useSWR<ThreadDetails>(
    threadId ? `${apiUrl}/threads/${threadId}` : null,
    fetcher
  );

  // Scroll to bottom when messages change
  useEffect(() => {
    if (messageContainerRef.current && data?.messages.length) {
      messageContainerRef.current.scrollTop = messageContainerRef.current.scrollHeight;
    }
  }, [data?.messages]);

  const handleSendMessage = async (message: string) => {
    setIsSending(true);

    try {
      if (!threadId) {
        // Create a new thread with the first message
        const response = await fetch(`${apiUrl}/threads`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ content: message }),
        });

        if (!response.ok) throw new Error('Failed to create thread');

        const data: CreateThreadResponse = await response.json();
        setThreadId(data.thread_id);
      } else {
        // Send message to existing thread
        const response = await fetch(`${apiUrl}/threads/${threadId}/message`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ content: message }),
        });

        if (!response.ok) throw new Error('Failed to send message');

        // Refresh the thread data to show the new messages
        await mutate();
      }
    } catch (error) {
      console.error("Error sending message:", error);
    } finally {
      setIsSending(false);
    }
  };

  let messages: JSX.Element[] = [];

  // Map the messages from the API response to UI components
  if (data) {
    messages = data.messages.map(message => {
      const { content } = message;

      if (content.role === "user") {
        return (
          <UserMessage key={message.id}>
            {content.message}
          </UserMessage>
        );
      } else if (content.role === "assistant") {
        return (
          <AssistantMessage key={message.id}>
            <ReactMarkdown>{content.message}</ReactMarkdown>
          </AssistantMessage>
        );
      } else if (content.role === "tool" && content.sidecar && content.sidecar !== "None") {
        return (
          <AssistantMessage key={message.id}>
            <strong>SQL:</strong>
            <p><code>{content.sidecar.SQLExecution[1]}</code></p>
          </AssistantMessage>
        );
      }
      return null;
    }).filter(Boolean) as JSX.Element[];
  }

  return (
    <div className="col-4 d-flex flex-column h-100 overflow-y-auto overflow-x-hidden">
      <nav className="navbar navbar-expand-lg position-sticky top-0 bg-primary-subtle bg-opacity-75">
        <div className="container-fluid">
          <a className="navbar-brand" href="#">
            <img src={logo} alt="logo" width="30" height="30" className="d-inline-block align-middle" />
            <span className="ms-1">BinaryBlackhole</span>
          </a>
        </div>
      </nav>

      <div
        ref={messageContainerRef}
        className="d-flex flex-column flex-grow-1 text-body overflow-y-auto"
      >
        {(isLoading && threadId) && (
          <div className="text-center p-3">
            <div className="spinner-border text-primary" role="status">
              <span className="visually-hidden">Loading...</span>
            </div>
          </div>
        )}

        {error && (
          <div className="alert alert-danger m-3" role="alert">
            Failed to load thread data
          </div>
        )}

        {messages}

        <div className="position-sticky bottom-0 mt-auto py-3 bg-primary-subtle bg-opacity-75">
          <SendMessageBox onSendMessage={handleSendMessage} isLoading={isSending} />
        </div>
      </div>
    </div>
  );
}

export default ChatBox;
