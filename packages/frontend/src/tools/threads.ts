// Thread management utilities
//
// This module provides reusable functions for creating and managing chat threads.
// It can be used across different components that need to create new threads
// or perform thread-related operations.
//
// Example usage:
//   import { createThread, archiveThread } from '../tools/threads';
//
//   // Create a new thread and navigate to it
//   const handleNewChat = async () => {
//     try {
//       const threadId = await createThread();
//       navigate(`/chats/${threadId}`);
//     } catch (error) {
//       console.error('Failed to create thread:', error);
//     }
//   };
//
//   // Archive an existing thread
//   const handleArchive = async (threadId: string) => {
//     try {
//       await archiveThread(threadId);
//       // Update UI state as needed
//     } catch (error) {
//       console.error('Failed to archive thread:', error);
//     }
//   };

export type CreateThreadResponse = {
  thread_id: string;
};

/**
 * Creates a new thread and returns the thread ID
 *
 * @returns Promise<string> - The ID of the newly created thread
 * @throws Error - If the API request fails or thread creation fails
 *
 * @example
 * ```typescript
 * const threadId = await createThread();
 * console.log('Created thread:', threadId);
 * ```
 */
export const createThread = async (): Promise<string> => {
  const apiUrl = import.meta.env.VITE_API_URL;
  if (!apiUrl) {
    throw new Error("API URL is not defined");
  }

  const response = await fetch(`${apiUrl}/threads`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({}),
  });

  if (!response.ok) {
    throw new Error("Failed to create thread");
  }

  const data: CreateThreadResponse = await response.json();
  return data.thread_id;
};

/**
 * Archives a thread, preventing new messages from being sent to it
 *
 * @param threadId - The ID of the thread to archive
 * @returns Promise<void> - Resolves when the thread is successfully archived
 * @throws Error - If the API request fails or archiving fails
 *
 * @example
 * ```typescript
 * await archiveThread("thread-123");
 * console.log('Thread archived successfully');
 * ```
 */
export const archiveThread = async (threadId: string): Promise<void> => {
  const apiUrl = import.meta.env.VITE_API_URL;
  if (!apiUrl) {
    throw new Error("API URL is not defined");
  }

  const response = await fetch(`${apiUrl}/threads/${threadId}/archive`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error("Failed to archive thread");
  }
};
