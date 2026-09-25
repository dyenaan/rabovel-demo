import { z } from "zod";

export const subscriptionSchema = z.object({
  assetId: z.string().min(1),
  amount: z
    .string()
    .min(1, "Amount is required.")
    .refine((v) => Number(v) > 0, "Amount must be greater than 0."),
  acknowledgeRisk: z.literal(true, { error: "You must acknowledge the offering risks." }),
});

export type SubscriptionFormValues = z.infer<typeof subscriptionSchema>;

export const redemptionSchema = z.object({
  assetId: z.string().min(1),
  quantity: z
    .string()
    .min(1, "Quantity is required.")
    .refine((v) => Number(v) > 0, "Quantity must be greater than 0."),
});

export type RedemptionFormValues = z.infer<typeof redemptionSchema>;
