// SWR fetcher function
export const fetcher = async (url: string) => {
  const apiUrl = import.meta.env.VITE_API_URL;
  if (!apiUrl) {
    throw new Error("API URL is not defined");
  }
  const res = await fetch(`${apiUrl}${url}`);
  if (!res.ok) {
    throw new Error("API request failed");
  }
  return res.json();
};

export async function* streamJsonLines<T>(
  url: string,
  init?: RequestInit,
): AsyncIterableIterator<T> {
  const apiUrl =
    import.meta.env.VITE_STREAMING_API_URL || import.meta.env.VITE_API_URL;
  if (!apiUrl) {
    throw new Error("API URL is not defined");
  }
  const response = await fetch(`${apiUrl}${url}`, init);

  if (!response.ok || !response.body) {
    throw new Error(`Network response was not ok, status: ${response.status}`);
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder("utf-8");

  let buffer = "";
  try {
    while (true) {
      const { value, done } = await reader.read();

      if (done) {
        if (buffer.trim()) {
          yield JSON.parse(buffer) as T;
        }
        break;
      }

      buffer += decoder.decode(value, { stream: true });

      let newlineIndex;
      while ((newlineIndex = buffer.indexOf("\n")) >= 0) {
        const line = buffer.slice(0, newlineIndex).trim();
        buffer = buffer.slice(newlineIndex + 1);

        if (line) {
          try {
            const parsed_line = JSON.parse(line) as T;
            console.log(`[got line]`, parsed_line);
            yield parsed_line;
          } catch (err) {
            console.warn("Failed to parse line as JSON:", line, err);
          }
        }
      }
    }
  } finally {
    reader.releaseLock();
  }
}
