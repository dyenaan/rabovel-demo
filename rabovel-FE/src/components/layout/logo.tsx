import Image from "next/image";
import Link from "next/link";

import { cn } from "@/lib/utils";

export function Logo({ className, href = "/" }: { className?: string; href?: string }) {
  return (
    <Link href={href} className={cn("inline-flex items-center", className)}>
      <Image
        src="/Rabovel.png"
        alt="Rabovel"
        width={604}
        height={131}
        priority
        className="h-7 w-auto"
      />
    </Link>
  );
}
