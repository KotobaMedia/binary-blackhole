# Page Structure

This directory contains the main page components for the application. The structure has been refactored to require explicit thread IDs for better separation of concerns.

## Page Components

### LandingPage (`LandingPage.tsx`)

- Entry point for the application
- Allows users to start a new conversation or explore data
- Sets initial message in global state and navigates to `/chat`

### NewChatPage (`NewChatPage.tsx`)

- Handles the `/chat` route (new chat creation)
- Automatically creates a new thread using the `createThread()` utility
- Shows loading state while creating the thread
- Redirects to `/chats/:threadId` once thread is created

### ChatMapPageWrapper (`ChatMapPageWrapper.tsx`)

- Wrapper component for the `/chats/:threadId` route
- Extracts threadId from the URL parameters
- Passes threadId as a prop to ChatMapPage
- Shows loading state if threadId is not available

### ChatMapPage (`ChatMapPage.tsx`)

- **Requires a `threadId` prop** - no longer handles thread creation
- Displays the chat interface with map
- Contains ChatBox and map components
- Focused solely on displaying and managing chat content

### ChatListPage (`ChatListPage.tsx`)

- Lists all existing chat threads
- Provides a "New Chat" button that uses the `createThread()` utility
- Demonstrates how the thread creation utility can be reused

### DataNavigatorPage (`DataNavigatorPage.tsx`)

- Data exploration interface
- Independent of chat functionality

## Thread Management

Thread creation and management is now centralized in `../tools/threads.ts`:

- `createThread()` - Creates a new thread and returns the thread ID
- `archiveThread(threadId)` - Archives an existing thread

This separation allows for:

- Reusable thread creation logic
- Consistent error handling
- Better type safety
- Easier testing and maintenance

## Routing Flow

1. **New Chat**: `/chat` → `NewChatPage` → creates thread → redirects to `/chats/:threadId`
2. **Existing Chat**: `/chats/:threadId` → `ChatMapPageWrapper` → `ChatMapPage`
3. **Chat List**: `/chats` → `ChatListPage` → can create new chat or navigate to existing

## Benefits

- **Clear separation of concerns**: Each component has a single responsibility
- **Reusable utilities**: Thread creation can be used anywhere in the app
- **Type safety**: All components that need threadId require it as a prop
- **Better error handling**: Centralized error handling for thread operations
- **Easier testing**: Components can be tested with mock threadIds
