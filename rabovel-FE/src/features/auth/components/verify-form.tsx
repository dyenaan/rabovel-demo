"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { zodResolver } from "@hookform/resolvers/zod";
import { Loader2 } from "lucide-react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Form, FormControl, FormField, FormItem, FormMessage } from "@/components/ui/form";
import { mockDelay } from "@/lib/api/mock-delay";
import { verifyCodeSchema, type VerifyCodeFormValues } from "../schemas/auth.schema";
import { OtpInput } from "./otp-input";

const RESEND_COOLDOWN_SECONDS = 30;

export function VerifyForm() {
  const router = useRouter();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isResending, setIsResending] = useState(false);
  const [cooldown, setCooldown] = useState(RESEND_COOLDOWN_SECONDS);

  const form = useForm<VerifyCodeFormValues>({
    resolver: zodResolver(verifyCodeSchema),
    defaultValues: { code: "" },
  });

  useEffect(() => {
    if (cooldown <= 0) return;
    const timer = setInterval(() => setCooldown((s) => s - 1), 1000);
    return () => clearInterval(timer);
  }, [cooldown]);

  async function onSubmit() {
    setIsSubmitting(true);
    await mockDelay(600);
    setIsSubmitting(false);
    toast.success("Email verified.");
    router.push("/onboarding");
  }

  async function handleResend() {
    setIsResending(true);
    await mockDelay(500);
    setIsResending(false);
    setCooldown(RESEND_COOLDOWN_SECONDS);
    toast.success("Verification code resent.");
  }

  return (
    <Card className="shadow-lg">
      <CardHeader>
        <CardTitle className="text-2xl tracking-tight">Verify your email</CardTitle>
        <CardDescription>
          We sent a 6-digit verification code to your email. Enter it below to continue.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4" noValidate>
            <FormField
              control={form.control}
              name="code"
              render={({ field, fieldState }) => (
                <FormItem>
                  <FormControl>
                    <OtpInput
                      value={field.value}
                      onChange={field.onChange}
                      disabled={isSubmitting}
                      aria-invalid={!!fieldState.error}
                    />
                  </FormControl>
                  <FormMessage className="text-center" />
                </FormItem>
              )}
            />
            <Button type="submit" className="w-full" disabled={isSubmitting}>
              {isSubmitting && <Loader2 className="size-4 animate-spin" aria-hidden="true" />}
              {isSubmitting ? "Verifying…" : "Verify"}
            </Button>
            <Button
              type="button"
              variant="ghost"
              className="w-full"
              disabled={isSubmitting || isResending || cooldown > 0}
              onClick={handleResend}
            >
              {isResending && <Loader2 className="size-4 animate-spin" aria-hidden="true" />}
              {cooldown > 0 ? `Resend code in ${cooldown}s` : "Resend code"}
            </Button>
          </form>
        </Form>
      </CardContent>
    </Card>
  );
}
