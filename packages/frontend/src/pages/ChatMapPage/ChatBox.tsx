import React, { JSX, useCallback, useState, useRef, useEffect } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import { QuestionCircleFill } from "react-bootstrap-icons";
import useSWR, { useSWRConfig } from "swr";
import { format as formatSQL } from "sql-formatter";
import { useLocation, useRoute } from "wouter";
import { layersAtom, SQLLayer } from "./atoms";
import { useSetAtom } from "jotai";
import Header from "../../components/Header";
import { fetcher, streamJsonLines } from "../../tools/api";

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

type CreateThreadResponse = {
  thread_id: string;
};

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

type SendMessageBoxProps = {
  onSendMessage: (message: string) => void;
  isLoading: boolean;
};

const SendMessageBox: React.FC<SendMessageBoxProps> = ({
  onSendMessage,
  isLoading,
}) => {
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Add effect to focus textarea when isLoading changes from true to false
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

      // Clear and reset the textarea
      textareaRef.current.value = "";
      textareaRef.current.style.height = "auto";
    },
    [onSendMessage, isLoading],
  );

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // Don't intercept key events during IME composition
    if (e.nativeEvent.isComposing || e.key === "Process") {
      return;
    }

    if (e.key === "Enter") {
      // Prevent submission when Shift is pressed (creating new line)
      if (e.shiftKey) {
        return;
      }

      e.preventDefault();

      if (
        textareaRef.current &&
        textareaRef.current.value.trim() &&
        !isLoading
      ) {
        // Call the form's submit handler to reuse the existing logic
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
      const maxHeight = 150; // maximum height in pixels
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

const useThreadId = () => {
  const [match, params] = useRoute("/chats/:threadId");
  return match ? params.threadId : null;
};
const useSetThreadId = () => {
  const [_, navigate] = useLocation();
  return useCallback(
    (id: string) => {
      navigate(`/chats/${id}`);
    },
    [navigate],
  );
};

const ChatBox: React.FC = () => {
  const threadId = useThreadId();
  const setThreadId = useSetThreadId();
  const setLayers = useSetAtom(layersAtom);
  const [isSending, setIsSending] = useState(false);
  const apiUrl = import.meta.env.VITE_API_URL;
  const messageContainerRef = useRef<HTMLDivElement>(null);

  const { mutate: globalMutate } = useSWRConfig();
  const {
    data: threadDetails,
    error,
    isLoading,
    mutate,
  } = useSWR<ThreadDetails>(threadId ? `/threads/${threadId}` : null, fetcher);

  // Hidden console API to archive the thread
  useEffect(() => {
    if (!threadId) return;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (window as any).__archiveThread = async () => {
      if (threadDetails?.archived) {
        console.warn("Thread is already archived");
        return;
      }

      const response = await fetch(`${apiUrl}/threads/${threadId}/archive`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
      });

      if (!response.ok) {
        console.error("Failed to archive thread");
        return;
      }

      mutate((prevData) => {
        if (!prevData) return prevData;
        return {
          ...prevData,
          archived: true,
        };
      }, false);
    };
    return () => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (window as any).__archiveThread = undefined;
    };
  }, [threadId, mutate, threadDetails?.archived, apiUrl]);

  // Scroll to bottom when messages change
  useEffect(() => {
    if (messageContainerRef.current && threadDetails?.messages.length) {
      messageContainerRef.current.scrollTop =
        messageContainerRef.current.scrollHeight;
    }
  }, [threadDetails?.messages]);

  const handleSendMessage = useCallback(
    async (message: string) => {
      setIsSending(true);

      try {
        let createdThreadId = threadId;
        if (!threadId) {
          const response = await fetch(`${apiUrl}/threads`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({}),
          });

          if (!response.ok) throw new Error("Failed to create thread");

          const data: CreateThreadResponse = await response.json();
          createdThreadId = data.thread_id;
          // If we created a new thread, update the URL
          setThreadId(createdThreadId);
        }
        if (!createdThreadId) throw new Error("Failed to create thread");
        const mutateKey = `/threads/${createdThreadId}`;

        const messageStream = streamJsonLines<Message>(
          `/threads/${createdThreadId}/message`,
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
          // Optimistically update the UI with the new message
          // Note we use globalMutate because it could be a new thread, and the
          // local mutate may not be available yet.
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

        // After we get the last message, everything should be in sync, but we'll run a mutate just in case.
        globalMutate(mutateKey);
      } catch (error) {
        console.error("Error sending message:", error);
        // If there was an error, revalidate to restore the correct state
        mutate();
      } finally {
        setIsSending(false);
      }
    },
    [threadId, globalMutate, apiUrl, setThreadId, mutate],
  );

  let messages: JSX.Element[] = [];

  // Set layers from data
  useEffect(() => {
    const messages = threadDetails?.messages;
    if (!messages) {
      setLayers([]);
      return;
    }
    const layers: SQLLayer[] = [];
    // TODO: this currently overwrites the layers every time
    // we get new data. We should probably merge them instead.
    // also, save the enabled state between refreshes
    for (const message of messages) {
      const { content } = message;
      if (content.role === "tool" && isSidecarSQLExecution(content.sidecar)) {
        const execution = content.sidecar.SQLExecution;
        layers.push({
          name: execution.name,
          sql: execution.sql,
          enabled: true,
        });
      }
    }
    setLayers(layers);
  }, [threadDetails?.messages, setLayers]);

  // Map the messages from the API response to UI components
  if (threadDetails) {
    messages = threadDetails.messages
      .map((message) => {
        const { content } = message;

        if (content.role === "user") {
          return <UserMessage key={message.id}>{content.message}</UserMessage>;
        } else if (content.role === "assistant") {
          return (
            <AssistantMessage key={message.id}>
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
            </AssistantMessage>
          );
        } else if (
          content.role === "tool" &&
          content.sidecar === "DatabaseLookup"
        ) {
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
      })
      .filter(Boolean) as JSX.Element[];
  }

  return (
    <div
      ref={messageContainerRef}
      className="d-flex flex-column h-100 overflow-y-auto overflow-x-hidden px-3"
    >
      <Header />
      <div className="d-flex flex-column flex-grow-1">
        {isLoading && threadId && (
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
      </div>

      {!threadDetails?.archived && (
        <div className="position-sticky bottom-0 mt-auto py-3 bg-body bg-opacity-75">
          <SendMessageBox
            onSendMessage={handleSendMessage}
            isLoading={isSending}
          />
        </div>
      )}
    </div>
  );
};

export default ChatBox;
