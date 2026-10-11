import { useMockApi } from "@/lib/env";
import { gatewayFetch } from "./gateway-fetch";
import { ApiRequestError, type ApiError, type ApiResponse } from "@/types";

export type RequestOptions = {
  params?: Record<string, string | number | boolean | undefined>;
  idempotencyKey?: string;
  signal?: AbortSignal;
  headers?: Record<string, string>;
};

function buildUrl(path: string, params?: RequestOptions["params"]): string {
  const query = new URLSearchParams();
  for (const [key, value] of Object.entries(params ?? {})) {
    if (value !== undefined) query.set(key, String(value));
  }
  return query.size ? `${path}?${query}` : path;
}

function createRequestId(): string {
  return typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `req_${Date.now()}_${Math.random().toString(36).slice(2)}`;
}

async function request<T>(
  method: "GET" | "POST" | "PATCH" | "PUT" | "DELETE",
  path: string,
  body?: unknown,
  options: RequestOptions = {},
): Promise<ApiResponse<T>> {
  const requestId = createRequestId();
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    "X-Request-Id": requestId,
    ...options.headers,
  };

  if (options.idempotencyKey) headers["Idempotency-Key"] = options.idempotencyKey;

  let response: Response;
  try {
    response = await gatewayFetch(buildUrl(path, options.params), {
      method,
      headers,
      body: body !== undefined ? JSON.stringify(body) : undefined,
      signal: options.signal,
    });
  } catch {
    throw new ApiRequestError({
      code: "NETWORK_ERROR",
      message: "Unable to reach the server. Check your connection and try again.",
      requestId,
    });
  }

  let payload: unknown = null;
  const text = await response.text();
  if (text) {
    try {
      payload = JSON.parse(text);
    } catch {
      payload = null;
    }
  }

  if (!response.ok) {
    const errorPayload = (payload as { error?: ApiError } | null)?.error;
    throw new ApiRequestError(
      {
        code: errorPayload?.code ?? `HTTP_${response.status}`,
        message:
          errorPayload?.message ??
          "Something went wrong processing your request. Please try again.",
        details: errorPayload?.details,
        requestId: errorPayload?.requestId ?? requestId,
      },
      response.status,
    );
  }

  return (payload as ApiResponse<T>) ?? { data: undefined as T };
}

export const apiClient = {
  get: <T>(path: string, options?: RequestOptions) =>
    request<T>("GET", path, undefined, options),
  post: <T>(path: string, body?: unknown, options?: RequestOptions) =>
    request<T>("POST", path, body, options),
  patch: <T>(path: string, body?: unknown, options?: RequestOptions) =>
    request<T>("PATCH", path, body, options),
  put: <T>(path: string, body?: unknown, options?: RequestOptions) =>
    request<T>("PUT", path, body, options),
  delete: <T>(path: string, options?: RequestOptions) =>
    request<T>("DELETE", path, undefined, options),
};

export function createIdempotencyKey(): string {
  return typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `idem_${Date.now()}_${Math.random().toString(36).slice(2)}`;
}

export const isMockApiEnabled = useMockApi;

/**
 * Real mode without a WebSocket stream polls live data (prices, book, tape,
 * portfolio) on the backend's 5-second price tick.
 */
export const livePollInterval: number | false =
  !useMockApi && !env.NEXT_PUBLIC_WS_URL ? 5_000 : false;
