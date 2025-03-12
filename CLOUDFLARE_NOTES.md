# Using Cloudflare

Right now, the main CDN for BBH is Cloudflare. Cloudflare proxies the requests between the user and the buffered and streaming Lambda URLs. The worker should have this code:

```javascript
addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

// Define your endpoints
const bufferedApi = '<buffered API Lambda URL hostname>'
const streamingApi = '<streaming API Lambda URL hostname>'

// Define your regex patterns
const streamingURLs = [
  /\/threads\/[a-zA-Z0-9]+\/message$/,
  // Add more regexes as needed
]

async function handleRequest(request) {
  const originalUrl = new URL(request.url)

  // Determine which API endpoint to use
  const isStreaming = streamingURLs.some(regex => regex.test(originalUrl.pathname))
  const newHost = isStreaming ? streamingApi : bufferedApi
  originalUrl.hostname = newHost

  const modifiedRequest = new Request(originalUrl.toString(), request)

  // Ensure the Host header matches the Lambda URL's hostname.
  modifiedRequest.headers.set('Host', newHost)

  return fetch(modifiedRequest)
}
```
