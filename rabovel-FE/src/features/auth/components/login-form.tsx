"use client";

import { useState } from "react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { zodResolver } from "@hookform/resolvers/zod";
import { Info, Loader2 } from "lucide-react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { mockLogin } from "../api/mock-auth";
import { fetchCurrentUser, homeForRole, loginWithPassword } from "../api/backend-auth";
import { useMockApi } from "@/lib/env";
import { useAuth } from "../hooks/use-auth";
import { loginSchema, type LoginFormValues } from "../schemas/auth.schema";
import { PasswordInput } from "./password-input";

/**
 * Resolves `path` against the current origin the same way a browser
 * navigation would (so backslash/percent tricks like "/\evil.com", which
 * WHATWG URL parsing treats as "//evil.com", are caught) and only returns it
 * if that resolution stays on this origin.
 */
function getSafeRedirect(path: string | null): string | null {
  if (!path) return null;
  try {
    const resolved = new URL(path, window.location.origin);
    if (resolved.origin !== window.location.origin) return null;
    return resolved.pathname + resolved.search + resolved.hash;
  } catch {
    return null;
  }
}

export function LoginForm() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { login } = useAuth();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const form = useForm<LoginFormValues>({
    resolver: zodResolver(loginSchema),
    defaultValues: { email: "", password: "" },
  });

  async function onSubmit(values: LoginFormValues) {
    setIsSubmitting(true);
    try {
      const user = useMockApi
        ? await mockLogin(values.email, values.password)
        : await loginWithPassword(values.email, values.password).then(async ({ access_token }) => ({
            user: await fetchCurrentUser(access_token),
            token: access_token,
          }));
      const authUser = "user" in user ? user.user : user;
      const token = "token" in user ? user.token : `mock_token_${authUser.id}`;
      login(authUser, token);
      toast.success(`Welcome back, ${authUser.name.split(" ")[0]}.`);
      const redirect = getSafeRedirect(searchParams.get("redirect"));
      router.push(redirect ?? homeForRole(authUser.role));
    } catch (error) {
      form.setError("password", {
        message: error instanceof Error ? error.message : "Unable to sign in.",
      });
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Card className="shadow-lg">
      <CardHeader>
        <CardTitle className="text-2xl tracking-tight">Log in to Rabovel</CardTitle>
        <CardDescription>
          Access your institutional dashboard, portfolio, and trading terminal.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4" noValidate>
            <FormField
              control={form.control}
              name="email"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Email</FormLabel>
                  <FormControl>
                    <Input
                      type="email"
                      placeholder="you@institution.com"
                      autoComplete="email"
                      autoFocus
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="password"
              render={({ field }) => (
                <FormItem>
                  <div className="flex items-center justify-between">
                    <FormLabel>Password</FormLabel>
                    <Link href="/forgot-password" className="text-xs font-medium text-primary hover:underline">
                      Forgot password?
                    </Link>
                  </div>
                  <FormControl>
                    <PasswordInput autoComplete="current-password" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <Button type="submit" className="w-full" disabled={isSubmitting}>
              {isSubmitting && <Loader2 className="size-4 animate-spin" aria-hidden="true" />}
              {isSubmitting ? "Signing in…" : "Log in"}
            </Button>
          </form>
        </Form>

        <div className="mt-6 flex gap-2 rounded-md border bg-muted/40 p-3 text-xs text-muted-foreground">
          <Info className="mt-0.5 size-3.5 shrink-0" aria-hidden="true" />
          <div>
            <p className="font-medium text-foreground">Investor and issuer access</p>
            {useMockApi ? (
              <>
                <p className="mt-1"><span className="font-medium text-foreground">Investor:</span> investor@rabovel.com</p>
                <p><span className="font-medium text-foreground">Issuer:</span> issuer@rabovel.com</p>
                <p>Password for both: password123</p>
              </>
            ) : (
              <p className="mt-1">
                Investors use their registered email. Issuers use the issuer email configured by the demo operator and are taken to the issuance workspace.
              </p>
            )}
          </div>
        </div>

        <p className="mt-6 text-center text-sm text-muted-foreground">
          Don&apos;t have an account?{" "}
          <Link href="/register" className="font-medium text-primary hover:underline">
            Request access
          </Link>
        </p>
      </CardContent>
    </Card>
  );
}
