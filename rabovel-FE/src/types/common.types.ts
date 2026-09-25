export type ApiResponse<T> = {
  data: T;
  meta?: {
    requestId?: string;
    nextCursor?: string;
    total?: number;
  };
};

export type ApiError = {
  code: string;
  message: string;
  details?: unknown;
  requestId?: string;
};

export class ApiRequestError extends Error {
  code: string;
  details?: unknown;
  requestId?: string;
  status?: number;

  constructor(error: ApiError, status?: number) {
    super(error.message);
    this.name = "ApiRequestError";
    this.code = error.code;
    this.details = error.details;
    this.requestId = error.requestId;
    this.status = status;
  }
}

export type Blockchain =
  | "ETHEREUM"
  | "POLYGON"
  | "ARBITRUM"
  | "BASE"
  | "SOLANA"
  | "AVALANCHE";

export type TokenStandard = "ERC-20" | "ERC-1400" | "ERC-3643" | "SPL";

export type UserRole =
  | "INVESTOR"
  | "ADMIN"
  | "COMPLIANCE"
  | "OPERATIONS"
  | "ISSUER"
  | "SUPPORT";

export type PaginatedParams = {
  cursor?: string;
  limit?: number;
};
