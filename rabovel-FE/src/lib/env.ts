import { z } from "zod";

const optionalUrl = z.preprocess(
  (value) => value === "" ? undefined : value,
  z.url().optional(),
);

const envSchema = z.object({
  NEXT_PUBLIC_API_BASE_URL: optionalUrl,
  NEXT_PUBLIC_WS_URL: optionalUrl,
  NEXT_PUBLIC_ENVIRONMENT: z.enum(["development", "staging", "production"]).default("development"),
  NEXT_PUBLIC_USE_MOCK_API: z
    .enum(["true", "false"])
    .default("true")
    .transform((value) => value === "true"),
}).superRefine((value, context) => {
  if (!value.NEXT_PUBLIC_USE_MOCK_API && !value.NEXT_PUBLIC_API_BASE_URL) {
    context.addIssue({
      code: "custom",
      path: ["NEXT_PUBLIC_API_BASE_URL"],
      message: "Required when NEXT_PUBLIC_USE_MOCK_API=false",
    });
  }
});

const parsed = envSchema.safeParse({
  NEXT_PUBLIC_API_BASE_URL: process.env.NEXT_PUBLIC_API_BASE_URL,
  NEXT_PUBLIC_WS_URL: process.env.NEXT_PUBLIC_WS_URL,
  NEXT_PUBLIC_ENVIRONMENT: process.env.NEXT_PUBLIC_ENVIRONMENT,
  NEXT_PUBLIC_USE_MOCK_API: process.env.NEXT_PUBLIC_USE_MOCK_API,
});

if (!parsed.success) {
  console.error(
    "Invalid environment configuration:",
    z.treeifyError(parsed.error),
  );
  throw new Error(
    "Invalid environment configuration. Check .env against .env.example.",
  );
}

export const env = {
  ...parsed.data,
  // The refinement above guarantees this in real API mode. The fallback is
  // never contacted in mock mode, but keeps callers' type simple.
  NEXT_PUBLIC_API_BASE_URL: parsed.data.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080",
};

export const useMockApi = env.NEXT_PUBLIC_USE_MOCK_API;
