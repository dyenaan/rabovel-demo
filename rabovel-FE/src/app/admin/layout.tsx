import { AdminBottomNav } from "@/components/layout/admin-bottom-nav";
import { AdminHeader } from "@/components/layout/admin-header";
import { AdminSidebar } from "@/components/layout/admin-sidebar";
import { AccessRestricted } from "@/features/auth/components/access-restricted";
import { RoleGuard } from "@/features/auth/components/role-guard";

export default function AdminLayout({ children }: LayoutProps<"/admin">) {
  return (
    <div className="flex min-h-screen">
      <AdminSidebar />
      <div className="flex min-w-0 flex-1 flex-col">
        <AdminHeader />
        <main className="flex-1 pb-16 md:pb-0">
          <RoleGuard allow={["ADMIN", "COMPLIANCE", "OPERATIONS"]} fallback={<AccessRestricted />}>
            {children}
          </RoleGuard>
        </main>
      </div>
      <AdminBottomNav />
    </div>
  );
}
