import React, { useState } from "react";
import { useLocation } from "wouter";
import { createThread } from "../tools/threads";

const LandingPage: React.FC = () => {
  const [, setLocation] = useLocation();
  const [conversationInput, setConversationInput] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleExploreData = () => {
    setLocation("/data");
  };

  const handleConversationSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (conversationInput.trim() && !isSubmitting) {
      setIsSubmitting(true);
      try {
        // Create a new thread first
        const threadId = await createThread();

        // Navigate to chat page with initial message in state
        setLocation(`/chats/${threadId}`, {
          state: { initialMessage: conversationInput.trim() },
        });
      } catch (error) {
        console.error("Failed to create thread:", error);
        // You might want to show an error message to the user here
      } finally {
        setIsSubmitting(false);
      }
    }
  };

  return (
    <div className="container-fluid vh-100 d-flex align-items-center justify-content-center">
      <div className="row justify-content-center w-100">
        <div className="col-md-8 col-lg-6 col-xl-4">
          <div className="card border-secondary">
            <div className="card-body text-center p-5">
              <h1 className="card-title mb-4">Binary Blackhole</h1>
              <p className="card-text mb-4 text-muted">
                データを探索し、空間情報との会話を始めましょう
              </p>

              <div className="d-grid gap-3">
                <button
                  onClick={handleExploreData}
                  className="btn btn-primary btn-lg"
                >
                  データを見る
                </button>

                <div className="text-muted small">または</div>

                <form onSubmit={handleConversationSubmit}>
                  <div className="input-group">
                    <input
                      type="text"
                      className="form-control form-control-lg"
                      placeholder="会話を始める..."
                      value={conversationInput}
                      onChange={(e) => setConversationInput(e.target.value)}
                    />
                    <button
                      type="submit"
                      className="btn btn-outline-secondary btn-lg"
                      disabled={!conversationInput.trim() || isSubmitting}
                    >
                      {isSubmitting ? (
                        <span
                          className="spinner-border spinner-border-sm"
                          role="status"
                          aria-hidden="true"
                        ></span>
                      ) : (
                        "Go"
                      )}
                    </button>
                  </div>
                </form>
              </div>

              <div className="mt-4 pt-3 border-top border-secondary">
                <small className="text-muted">
                  <strong>EXPERIMENTAL</strong> •
                  すべてのクエリはログされています •
                  <a
                    href="https://github.com/KotobaMedia/binary-blackhole"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-decoration-none ms-1"
                  >
                    GitHub
                  </a>
                </small>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default LandingPage;
