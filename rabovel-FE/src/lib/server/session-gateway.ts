// Server-only gateway core. Imported only by the route handler and Node tests.
import { isIP } from "node:net";
import { randomUUID } from "node:crypto";

export type GatewayConfig = {
  backendOrigin: string;
  publicOrigin: string;
  secret: string;
  production: boolean;
};

const routes: [RegExp, string[]][] = [
  [/^\/auth\/(login|register|logout)$/, ["POST"]],
  [/^\/auth\/me$/, ["GET"]],
  [/^\/instruments$/, ["GET"]],
  [/^\/assets(?:\/[A-Za-z0-9_-]+(?:\/nav-history)?)?$/, ["GET"]],
  [/^\/markets(?:\/[A-Za-z0-9_-]+(?:\/(order-book|price-history|trades))?)?$/, ["GET"]],
  [/^\/portfolio\/(summary|holdings|performance|activity)$/, ["GET"]],
  [/^\/issuer\/overview$/, ["GET"]],
  [/^\/issuer\/onboarding$/, ["POST"]],
  [/^\/issuer\/assets$/, ["GET", "POST"]],
  [/^\/issuer\/assets\/ast_[a-f0-9]+$/, ["GET", "PATCH", "DELETE"]],
  [/^\/issuer\/assets\/ast_[a-f0-9]+\/(setup|inventory)$/, ["GET", "POST"]],
  [/^\/issuer\/assets\/ast_[a-f0-9]+\/(submit|backing|listing|image-upload|image-upload\/confirm)$/, ["POST"]],
  [/^\/payment-assets\/cngn$/, ["GET", "POST"]],
  [/^\/onboarding\/status$/, ["GET"]],
  [/^\/onboarding\/terms$/, ["POST"]],
  [/^\/wallet\/(challenges|solana\/challenges|link|authorize)$/, ["POST"]],
  [/^\/wallets$/, ["GET"]],
  [/^\/wallet\/payment-assets\/cngn$/, ["GET", "POST"]],
  [/^\/kyc\/cases$/, ["POST"]],
  [/^\/(portfolio|investor\/catalog)$/, ["GET"]],
  [/^\/investor\/(quote|purchase\/prepare|purchase\/submit)$/, ["POST"]],
  [/^\/orders$/, ["GET", "POST"]],
  [/^\/orders\/[A-Za-z0-9_-]+$/, ["GET", "DELETE"]],
  [/^\/tokenization\/requests$/, ["POST"]],
  [/^\/tokenization\/requests\/[A-Za-z0-9_-]+$/, ["GET"]],
];
const TOKEN = /^[a-f0-9]{64}$/;
const BODY_LIMIT = 16 * 1024;

export function gatewayConfig(environment: NodeJS.ProcessEnv): GatewayConfig {
  const production = environment.NODE_ENV === "production";
  function origin(value: string | undefined): string {
    const url = new URL(value ?? "");
    if (url.username || url.password || url.pathname !== "/" || url.search || url.hash ||
        (url.protocol !== "https:" && !(url.protocol === "http:" && !production &&
          ["localhost", "127.0.0.1", "[::1]"].includes(url.hostname)))) {
      throw new Error("Invalid gateway origin");
    }
    return url.origin;
  }
  const secret = environment.RABOVEL_GATEWAY_SECRET ?? "";
  if (!/^[a-fA-F0-9]{64}$/.test(secret)) throw new Error("Invalid gateway secret");
  return {
    backendOrigin: origin(environment.RABOVEL_API_URL),
    publicOrigin: origin(environment.RABOVEL_PUBLIC_ORIGIN),
    secret,
    production,
  };
}

function cookieName(config: GatewayConfig): string {
  return config.production ? "__Host-rabovel-session" : "rabovel-session";
}

function cookie(config: GatewayConfig, token = "", age = 0): string {
  return `${cookieName(config)}=${token}; Path=/; HttpOnly; SameSite=Lax; Max-Age=${age}${config.production ? "; Secure" : ""}`;
}

async function readLimited(stream: ReadableStream<Uint8Array> | null, limit: number): Promise<string> {
  if (!stream) return "";
  const reader = stream.getReader();
  let size = 0;
  const chunks: Uint8Array[] = [];
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      size += value.byteLength;
      if (size > limit) {
        await reader.cancel();
        throw new Error("body_limit");
      }
      chunks.push(value);
    }
  } finally {
    reader.releaseLock();
  }
  return Buffer.concat(chunks).toString("utf8");
}

export async function handleGateway(
  request: Request,
  config: GatewayConfig,
  fetchBackend: typeof fetch = fetch,
): Promise<Response> {
  const headers = new Headers({ "Cache-Control": "no-store", "X-Content-Type-Options": "nosniff" });
  const requestId = randomUUID();
  headers.set("X-Request-Id", requestId);
  const error = (status: number, code: string, message: string) =>
    Response.json({ code, message }, { status, headers });
  const incoming = new URL(request.url);
  const path = incoming.pathname.slice("/api".length);
  if (!incoming.pathname.startsWith("/api/") || incoming.search.length > 2048 || /[%\\]/.test(path)) {
    return error(404, "NOT_FOUND", "Unknown API route.");
  }
  const allowed = routes.find(([pattern]) => pattern.test(path));
  if (!allowed) return error(404, "NOT_FOUND", "Unknown API route.");
  if (!allowed[1].includes(request.method)) return error(405, "METHOD_NOT_ALLOWED", "Unsupported method.");

  const unsafe = request.method !== "GET";
  if (unsafe && request.headers.get("origin") !== config.publicOrigin) {
    return error(403, "INVALID_ORIGIN", "Request origin is not allowed.");
  }
  if (unsafe && request.headers.get("content-type")?.split(";")[0].trim().toLowerCase() !== "application/json") {
    return error(415, "INVALID_CONTENT_TYPE", "Use application/json.");
  }

  const sessionCreation = path === "/auth/login" || path === "/auth/register";
  const logout = path === "/auth/logout";
  if (logout) headers.set("Set-Cookie", cookie(config));
  const session = request.headers.get("cookie")?.split(";")
    .map((part) => part.trim()).find((part) => part.startsWith(`${cookieName(config)}=`))
    ?.slice(cookieName(config).length + 1);
  const publicRoute = sessionCreation || path === "/instruments";
  if (!publicRoute && (!session || !TOKEN.test(session))) {
    headers.set("Set-Cookie", cookie(config));
    return error(401, "unauthorized", "Sign in to continue.");
  }

  // Vercel overwrites this header at its edge. Never forward caller-supplied
  // gateway/auth headers; build a fresh allowlisted set for the fixed backend.
  const ip = config.production ? request.headers.get("x-vercel-forwarded-for") : "127.0.0.1";
  if (!ip || !isIP(ip)) return error(400, "INVALID_CLIENT_IP", "Client address is unavailable.");
  const upstreamHeaders = new Headers({
    "Content-Type": "application/json",
    "X-Request-Id": requestId,
    "X-Rabovel-Gateway-Secret": config.secret,
    "X-Rabovel-Client-IP": ip,
  });
  if (!sessionCreation && session && TOKEN.test(session)) upstreamHeaders.set("Authorization", `Bearer ${session}`);
  const idempotency = request.headers.get("idempotency-key");
  if (idempotency && /^[A-Za-z0-9_-]{1,128}$/.test(idempotency)) upstreamHeaders.set("Idempotency-Key", idempotency);

  let body: string | undefined;
  try {
    if (unsafe) body = await readLimited(request.body, BODY_LIMIT) || undefined;
  } catch {
    return error(413, "PAYLOAD_TOO_LARGE", "Request exceeds 16 KiB.");
  }

  const chainOperation = unsafe && /\/(setup|inventory|prepare|submit|cngn)$/.test(path);
  const signal = AbortSignal.timeout(chainOperation ? 280_000 : 30_000);
  try {
    const upstream = await fetchBackend(`${config.backendOrigin}${path}${incoming.search}`, {
      method: request.method, headers: upstreamHeaders, body,
      redirect: "manual", cache: "no-store", signal,
    });
    if (upstream.status >= 300 && upstream.status < 400) {
      await upstream.body?.cancel();
      return error(502, "UPSTREAM_ERROR", "The backend returned an unexpected redirect.");
    }
    if (upstream.status >= 500) {
      await upstream.body?.cancel();
      return error(503, "SERVICE_UNAVAILABLE", "The service is temporarily unavailable.");
    }
    const text = await readLimited(upstream.body, 2 * 1024 * 1024);
    if (sessionCreation && upstream.ok) {
      const payload = JSON.parse(text) as { access_token?: unknown; expires_in?: unknown };
      if (typeof payload.access_token !== "string" || !TOKEN.test(payload.access_token) ||
          typeof payload.expires_in !== "number" || !Number.isInteger(payload.expires_in) ||
          payload.expires_in < 1 || payload.expires_in > 900) throw new Error("invalid_session");
      headers.set("Set-Cookie", cookie(config, payload.access_token, payload.expires_in));
      // No bearer credential is returned to browser JavaScript.
      return Response.json({ expires_in: payload.expires_in }, { status: upstream.status, headers });
    }
    if (upstream.status === 401 && !sessionCreation) headers.set("Set-Cookie", cookie(config));
    const retryAfter = upstream.headers.get("retry-after");
    if (retryAfter && /^\d{1,8}$/.test(retryAfter)) headers.set("Retry-After", retryAfter);
    headers.set("Content-Type", "application/json");
    return new Response(upstream.status === 204 ? null : text, { status: upstream.status, headers });
  } catch {
    return signal.aborted
      ? error(504, "UPSTREAM_TIMEOUT", "The request timed out. Check operation status before retrying.")
      : error(503, "SERVICE_UNAVAILABLE", "The service is temporarily unavailable.");
  }
}
