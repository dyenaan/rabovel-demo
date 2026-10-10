/** Server-side policy builder. No secrets belong in browser headers. */
export function browserHeaders(production: boolean): { key: string; value: string }[] {
  const headers = [
    { key: "X-Content-Type-Options", value: "nosniff" },
    { key: "X-Frame-Options", value: "DENY" },
    { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
    { key: "Permissions-Policy", value: "camera=(), microphone=(), geolocation=(), payment=(), usb=()" },
    // Wallet extensions and sign-in flows may use popups.
    { key: "Cross-Origin-Opener-Policy", value: "same-origin-allow-popups" },
  ];
  if (production) headers.push({ key: "Strict-Transport-Security", value: "max-age=31536000" });
  return headers;
}

export function contentSecurityPolicy(nonce: string, environment: NodeJS.ProcessEnv): string {
  if (!/^[A-Za-z0-9+/=]{16,128}$/.test(nonce)) throw new Error("Invalid CSP nonce");
  const production = environment.NODE_ENV === "production";
  const connections = ["'self'"];
  if (environment.RABOVEL_STORAGE_ORIGIN) {
    const url = new URL(environment.RABOVEL_STORAGE_ORIGIN);
    if (url.protocol !== "https:" || url.username || url.password || url.pathname !== "/" || url.search || url.hash) {
      throw new Error("RABOVEL_STORAGE_ORIGIN must be an HTTPS origin");
    }
    connections.push(url.origin);
  }
  if (environment.NEXT_PUBLIC_WS_URL) {
    const url = new URL(environment.NEXT_PUBLIC_WS_URL);
    if (url.username || url.password || !(url.protocol === "wss:" || !production && url.protocol === "ws:" && ["localhost", "127.0.0.1", "[::1]"].includes(url.hostname))) {
      throw new Error("Invalid WebSocket security origin");
    }
    connections.push(url.origin);
  }
  // Development requires eval for React diagnostics and local HMR connections.
  if (!production) connections.push("ws://localhost:*", "ws://127.0.0.1:*");
  return [
    "default-src 'self'",
    `script-src 'self' 'nonce-${nonce}' 'strict-dynamic'${production ? "" : " 'unsafe-eval'"}`,
    // Existing Radix, charts and motion components use inline styles.
    "style-src 'self' 'unsafe-inline'",
    "img-src 'self' https: data: blob:",
    "font-src 'self'",
    `connect-src ${connections.join(" ")}`,
    "object-src 'none'", "base-uri 'none'", "form-action 'self'", "frame-ancestors 'none'", "frame-src 'none'",
    ...(production ? ["upgrade-insecure-requests"] : []),
  ].join("; ");
}
