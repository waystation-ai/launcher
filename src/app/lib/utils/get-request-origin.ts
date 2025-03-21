/**
 * Gets the correct origin URL for the request, taking into account proxies like ngrok
 * by checking X-Forwarded headers
 */
export function getRequestOrigin(request: Request): string {
  const url = new URL(request.url);
  
  // Check for forwarded host and protocol (set by proxies like ngrok)
  const forwardedHost = request.headers.get('x-forwarded-host');
  const forwardedProto = request.headers.get('x-forwarded-proto');
  
  if (forwardedHost && forwardedProto) {
    // We're behind a proxy, use the forwarded information
    return `${forwardedProto}://${forwardedHost}`;
  }
  
  // No proxy, use the original origin
  return url.origin;
}
