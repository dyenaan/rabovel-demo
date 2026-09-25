import { Logo } from "@/components/layout/logo";

export default function OnboardingLayout({ children }: LayoutProps<"/">) {
  return (
    <div className="flex min-h-screen flex-col bg-secondary/30">
      <header className="border-b bg-background px-4 py-4 sm:px-6">
        <Logo />
      </header>
      <main className="flex flex-1 justify-center px-4 py-10">
        <div className="w-full max-w-2xl">{children}</div>
      </main>
    </div>
  );
}
