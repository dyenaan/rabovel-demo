import { z } from "zod";

const optionalUrl = z.preprocess(
  (value) => value === "" ? undefined : value,
  z.url().optional(),
);

const envSchema = z.object({
  NEXT_PUBLIC_WS_URL: optionalUrl,
  NEXT_PUBLIC_ENVIRONMENT: z.enum(["development", "staging", "production"]).default("development"),
  NEXT_PUBLIC_USE_MOCK_API: z
    .enum(["true", "false"])
    .default(process.env.NODE_ENV === "production" ? "false" : "true")
    .transform((value) => value === "true"),
}).superRefine((value, context) => {
  if (process.env.NODE_ENV === "production" && value.NEXT_PUBLIC_USE_MOCK_API) {
    context.addIssue({ code: "custom", path: ["NEXT_PUBLIC_USE_MOCK_API"], message: "Mock authentication is disabled in production" });
  }
});

const parsed = envSchema.safeParse({
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

export const env = parsed.data;

export const useMockApi = env.NEXT_PUBLIC_USE_MOCK_API;
