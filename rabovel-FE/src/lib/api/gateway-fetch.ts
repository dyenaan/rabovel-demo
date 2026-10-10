/** Browser requests use the same-origin gateway; credentials never leave it. */
export function gatewayFetch(path: string, init?: RequestInit): Promise<Response> {
  if (!path.startsWith("/") || path.startsWith("//") || (/[%\\]/.test(path.split("?")[0]) || /[#\r\n]/.test(path))) {
    throw new Error("Invalid API path");
  }
  const headers = new Headers(init?.headers);
  headers.delete("Authorization");
  headers.delete("X-Rabovel-Gateway-Secret");
  headers.delete("X-Rabovel-Client-IP");
  if (init?.method && init.method !== "GET") headers.set("Content-Type", "application/json");
  return fetch(`/api${path}`, { ...init, headers, credentials: "same-origin", cache: "no-store" });
}
