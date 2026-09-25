import { z } from "zod";

export const orderFormSchema = z
  .object({
    side: z.enum(["BUY", "SELL"]),
    orderType: z.enum(["MARKET", "LIMIT", "STOP_LIMIT"]),
    limitPrice: z.string().optional(),
    quantity: z
      .string()
      .min(1, "Quantity is required.")
      .refine((value) => Number(value) > 0, "Quantity must be greater than 0."),
  })
  .refine(
    (data) =>
      data.orderType === "MARKET" || (!!data.limitPrice && Number(data.limitPrice) > 0),
    { message: "Price is required for this order type.", path: ["limitPrice"] },
  );

export type OrderFormValues = z.infer<typeof orderFormSchema>;
