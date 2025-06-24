import { atom } from "jotai";

export type ConversationState = {
  initialMessage: string | null;
  isInitialMessageSent: boolean;
};

export const conversationAtom = atom<ConversationState>({
  initialMessage: null,
  isInitialMessageSent: false,
});

// Helper atoms for easier access
export const initialMessageAtom = atom(
  (get) => get(conversationAtom).initialMessage,
  (get, set, message: string | null) => {
    set(conversationAtom, {
      ...get(conversationAtom),
      initialMessage: message,
    });
  },
);

export const isInitialMessageSentAtom = atom(
  (get) => get(conversationAtom).isInitialMessageSent,
  (get, set, sent: boolean) => {
    set(conversationAtom, {
      ...get(conversationAtom),
      isInitialMessageSent: sent,
    });
  },
);

// Action atom to set initial message and reset sent flag
export const setInitialMessageAtom = atom(null, (get, set, message: string) => {
  set(conversationAtom, {
    initialMessage: message,
    isInitialMessageSent: false,
  });
});

// Action atom to mark initial message as sent
export const markInitialMessageSentAtom = atom(null, (get, set) => {
  set(conversationAtom, {
    ...get(conversationAtom),
    isInitialMessageSent: true,
  });
});

// Action atom to clear conversation state
export const clearConversationStateAtom = atom(null, (get, set) => {
  set(conversationAtom, {
    initialMessage: null,
    isInitialMessageSent: false,
  });
});
