import React, { useCallback, useState, useRef, useEffect } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import { QuestionCircleFill } from "react-bootstrap-icons";
import useSWR, { useSWRConfig } from "swr";
import { format as formatSQL } from "sql-formatter";
import { layersAtom, SQLLayer } from "./atoms";
import { useSetAtom } from "jotai";
import Header from "../../components/Header";
import { fetcher, streamJsonLines } from "../../tools/api";
import { archiveThread } from "../../tools/threads";

// Types for the API response data
type Role = "user" | "assistant" | "system" | "tool";

type SQLExecutionDetails = {
  id: string;
  name: string;
  sql: string;
};

type ChatterMessageSidecar =
  | "None"
  | "DatabaseLookup"
  | {
      SQLExecution: SQLExecutionDetails;
    };
function isSidecarSQLExecution(
  sidecar?: ChatterMessageSidecar,
): sidecar is { SQLExecution: SQLExecutionDetails } {
  return typeof sidecar === "object" && sidecar && "SQLExecution" in sidecar;
}

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
  archived?: boolean;
};

// Optimistic message type for local state
type OptimisticMessage = {
  id: string; // Temporary ID for optimistic messages
  content: ChatterMessageView;
  isOptimistic: true;
  timestamp: number;
};

// Combined message type for rendering
type RenderMessage = Message | OptimisticMessage;

// Custom hook for managing chat state and operations
const useChatState = (threadId: string) => {
  const [optimisticMessages, setOptimisticMessages] = useState<
    OptimisticMessage[]
  >([]);
  const [isSending, setIsSending] = useState(false);
  const { mutate: globalMutate } = useSWRConfig();

  const {
    data: threadDetails,
    error,
    isLoading,
    mutate,
  } = useSWR<ThreadDetails>(`/threads/${threadId}`, fetcher);

  // Combine server messages with optimistic messages
  const allMessages: RenderMessage[] = [
    ...(threadDetails?.messages || []),
    ...optimisticMessages,
  ].sort((a, b) => {
    // Sort by timestamp for optimistic messages, by id for server messages
    const aTime = "timestamp" in a ? a.timestamp : a.id;
    const bTime = "timestamp" in b ? b.timestamp : b.id;
    return aTime - bTime;
  });

  const addOptimisticMessage = useCallback((content: ChatterMessageView) => {
    const optimisticMessage: OptimisticMessage = {
      id: `optimistic-${Date.now()}-${Math.random()}`,
      content,
      isOptimistic: true,
      timestamp: Date.now(),
    };
    setOptimisticMessages((prev) => [...prev, optimisticMessage]);
    return optimisticMessage.id;
  }, []);

  const removeOptimisticMessage = useCallback((id: string) => {
    setOptimisticMessages((prev) => prev.filter((msg) => msg.id !== id));
  }, []);

  const sendMessage = useCallback(
    async (message: string) => {
      setIsSending(true);

      // Add optimistic user message immediately
      const optimisticId = addOptimisticMessage({
        message,
        role: "user",
      });

      try {
        const mutateKey = `/threads/${threadId}`;

        // Remove optimistic message once we start getting real messages
        removeOptimisticMessage(optimisticId);

        const messageStream = streamJsonLines<Message>(
          `/threads/${threadId}/message`,
          {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              content: message,
            }),
          },
        );

        for await (const message of messageStream) {
          globalMutate<ThreadDetails>(
            mutateKey,
            (prevData) => {
              if (!prevData) return prevData;
              const newMessages = [...prevData.messages, message];
              return {
                ...prevData,
                messages: newMessages,
              };
            },
            false,
          );
        }

        globalMutate(mutateKey);
      } catch (error) {
        console.error("Error sending message:", error);
        // Remove optimistic message on error
        removeOptimisticMessage(optimisticId);
        mutate();
      } finally {
        setIsSending(false);
      }
    },
    [
      threadId,
      globalMutate,
      mutate,
      addOptimisticMessage,
      removeOptimisticMessage,
    ],
  );

  return {
    allMessages,
    isSending,
    isLoading,
    error,
    threadDetails,
    sendMessage,
    mutate,
  };
};

// Message rendering components
const AssistantMessage: React.FC<React.PropsWithChildren> = ({ children }) => {
  return (
    <div className="d-flex justify-content-start mb-2">
      <div className="p-2 rounded" style={{ maxWidth: "90%" }}>
        {children}
      </div>
    </div>
  );
};

const UserMessage: React.FC<React.PropsWithChildren> = ({ children }) => {
  return (
    <div className="d-flex justify-content-end mb-2">
      <div
        className="bg-primary text-white p-2 rounded"
        style={{ maxWidth: "90%" }}
      >
        {children}
      </div>
    </div>
  );
};

// Message renderer component
const MessageRenderer: React.FC<{ message: RenderMessage }> = ({ message }) => {
  const { content } = message;
  const isOptimistic = "isOptimistic" in message;

  if (content.role === "user") {
    return (
      <UserMessage key={message.id}>
        <div className="markdown-content">
          <ReactMarkdown
            remarkPlugins={[remarkMath, remarkGfm]}
            rehypePlugins={[rehypeKatex]}
            components={{
              table: ({ node: _node, ...props }) => (
                <table className="table" {...props} />
              ),
            }}
          >
            {content.message}
          </ReactMarkdown>
        </div>
        {isOptimistic && (
          <div className="small text-muted mt-1">
            <span
              className="spinner-border spinner-border-sm me-1"
              role="status"
              aria-hidden="true"
            ></span>
            送信中...
          </div>
        )}
      </UserMessage>
    );
  } else if (content.role === "assistant") {
    return (
      <AssistantMessage key={message.id}>
        <div className="markdown-content">
          <ReactMarkdown
            remarkPlugins={[remarkMath, remarkGfm]}
            rehypePlugins={[rehypeKatex]}
            components={{
              table: ({ node: _node, ...props }) => (
                <table className="table" {...props} />
              ),
            }}
          >
            {content.message}
          </ReactMarkdown>
        </div>
      </AssistantMessage>
    );
  } else if (content.role === "tool" && content.sidecar === "DatabaseLookup") {
    return (
      <AssistantMessage key={message.id}>
        <strong>データベース確認中...</strong>
      </AssistantMessage>
    );
  } else if (
    content.role === "tool" &&
    isSidecarSQLExecution(content.sidecar)
  ) {
    let sqlText = content.sidecar.SQLExecution.sql;
    try {
      sqlText = formatSQL(content.sidecar.SQLExecution.sql, {
        language: "postgresql",
        tabWidth: 2,
        keywordCase: "upper",
      });
    } catch (e) {
      console.error("Error formatting SQL:", e);
    }
    return (
      <AssistantMessage key={message.id}>
        <strong>SQL:</strong>
        <pre>
          <code>{sqlText}</code>
        </pre>
      </AssistantMessage>
    );
  }
  return null;
};

// Send message box component
type SendMessageBoxProps = {
  onSendMessage: (message: string) => void;
  isLoading: boolean;
};

const SendMessageBox: React.FC<SendMessageBoxProps> = ({
  onSendMessage,
  isLoading,
}) => {
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (!isLoading && textareaRef.current) {
      textareaRef.current.focus();
    }
  }, [isLoading]);

  const onSubmit = useCallback<React.FormEventHandler<HTMLFormElement>>(
    (e) => {
      e.preventDefault();
      if (!textareaRef.current || isLoading) return;

      const message = textareaRef.current.value.trim();
      if (!message) return;

      onSendMessage(message);

      textareaRef.current.value = "";
      textareaRef.current.style.height = "auto";
    },
    [onSendMessage, isLoading],
  );

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.nativeEvent.isComposing || e.key === "Process") {
      return;
    }

    if (e.key === "Enter") {
      if (e.shiftKey) {
        return;
      }

      e.preventDefault();

      if (
        textareaRef.current &&
        textareaRef.current.value.trim() &&
        !isLoading
      ) {
        const form = e.currentTarget.closest("form");
        if (form) {
          form.requestSubmit();
        }
      }
    }
  };

  const onInput = useCallback<React.FormEventHandler<HTMLTextAreaElement>>(
    (e) => {
      const el = e.currentTarget;
      const maxHeight = 150;
      el.style.height = "auto";
      if (el.scrollHeight < maxHeight) {
        el.style.height = `${el.scrollHeight}px`;
        el.style.overflowY = "hidden";
      } else {
        el.style.height = `${maxHeight}px`;
        el.style.overflowY = "auto";
      }
    },
    [],
  );

  return (
    <form onSubmit={onSubmit}>
      <div className="input-group">
        <textarea
          ref={textareaRef}
          className="form-control"
          placeholder="何を調べましょうか？"
          rows={1}
          style={{ overflow: "hidden", resize: "none", maxHeight: "150px" }}
          onInput={onInput}
          onKeyDown={handleKeyDown}
          disabled={isLoading}
        />
        <button className="btn btn-primary" type="submit" disabled={isLoading}>
          {isLoading ? (
            <span
              className="spinner-border spinner-border-sm"
              role="status"
              aria-hidden="true"
            ></span>
          ) : (
            <QuestionCircleFill className="align-baseline" title="Submit" />
          )}
        </button>
      </div>
    </form>
  );
};

// Main ChatBox component
type ChatBoxProps = {
  threadId: string;
};

const ChatBox: React.FC<ChatBoxProps> = ({ threadId }) => {
  const setLayers = useSetAtom(layersAtom);
  const messageContainerRef = useRef<HTMLDivElement>(null);
  const hasSentInitialMessage = useRef(false);

  const {
    allMessages,
    isSending,
    isLoading,
    error,
    threadDetails,
    sendMessage,
    mutate,
  } = useChatState(threadId);

  // Scroll to bottom when messages change
  useEffect(() => {
    if (messageContainerRef.current && allMessages.length) {
      messageContainerRef.current.scrollTop =
        messageContainerRef.current.scrollHeight;
    }
  }, [allMessages]);

  // Set layers from data
  useEffect(() => {
    const messages = threadDetails?.messages;
    if (!messages) {
      setLayers([]);
      return;
    }
    const layers: SQLLayer[] = [];
    for (const message of messages) {
      const { content } = message;
      if (content.role === "tool" && isSidecarSQLExecution(content.sidecar)) {
        const execution = content.sidecar.SQLExecution;
        layers.push({
          id: execution.id,
          name: execution.name,
          sql: execution.sql,
          enabled: true,
        });
      }
    }
    setLayers(layers);
  }, [threadDetails?.messages, setLayers]);

  // Hidden console API to archive the thread
  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (window as any).__archiveThread = async () => {
      if (threadDetails?.archived) {
        console.warn("Thread is already archived");
        return;
      }

      try {
        await archiveThread(threadId);
        mutate((prevData) => {
          if (!prevData) return prevData;
          return {
            ...prevData,
            archived: true,
          };
        }, false);
      } catch (error) {
        console.error("Failed to archive thread:", error);
      }
    };
    return () => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (window as any).__archiveThread = undefined;
    };
  }, [threadId, mutate, threadDetails?.archived]);

  // Get initial message from navigation state
  const getInitialMessage = () => {
    const state = history.state as { initialMessage?: string } | null;
    return state?.initialMessage || null;
  };

  // Reset initial message flag when threadId changes
  useEffect(() => {
    hasSentInitialMessage.current = false;
  }, [threadId]);

  // Simple initial message handling - send once when ready, then clear
  useEffect(() => {
    const initialMessage = getInitialMessage();

    if (
      initialMessage &&
      !hasSentInitialMessage.current &&
      threadDetails &&
      !threadDetails.archived
    ) {
      hasSentInitialMessage.current = true;
      sendMessage(initialMessage);
      // Clear the state after sending
      history.replaceState(null, "", window.location.href);
    }
  }, [threadDetails, sendMessage]);

  return (
    <div
      ref={messageContainerRef}
      className="d-flex flex-column h-100 overflow-y-auto overflow-x-hidden px-3"
    >
      <Header />

      <div className="d-flex flex-column flex-grow-1">
        {isLoading && (
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

        {allMessages.map((message) => (
          <MessageRenderer key={message.id} message={message} />
        ))}
      </div>

      {!threadDetails?.archived && (
        <div className="position-sticky bottom-0 mt-auto py-3 bg-body bg-opacity-75">
          <SendMessageBox onSendMessage={sendMessage} isLoading={isSending} />
        </div>
      )}
    </div>
  );
};

export default ChatBox;
