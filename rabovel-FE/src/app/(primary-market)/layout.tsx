import { AppBottomNav } from "@/components/layout/app-bottom-nav";
import { AppHeader } from "@/components/layout/app-header";
import { AppSidebar } from "@/components/layout/app-sidebar";
import { MobileNavigation } from "@/components/layout/mobile-navigation";
import { AccessRestricted } from "@/features/auth/components/access-restricted";
import { RoleGuard } from "@/features/auth/components/role-guard";

export default function PrimaryMarketLayout({ children }: LayoutProps<"/">) {
  return (
    <div className="flex min-h-screen">
      <AppSidebar />
      <MobileNavigation />
      <div className="flex min-w-0 flex-1 flex-col">
        <AppHeader />
        <main className="flex-1 pb-16 md:pb-0">
          <RoleGuard allow={["INVESTOR"]} fallback={<AccessRestricted />}>{children}</RoleGuard>
        </main>
      </div>
      <AppBottomNav />
    </div>
  );
}
