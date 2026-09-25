import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * Whether a nav link's href matches the current pathname, treating it as active
 * for exact matches and (unless `exact` is set) for nested sub-routes too.
 * Pass `exact: true` for a section's own root link (e.g. "/admin") so it doesn't
 * stay highlighted while a sibling sub-route like "/admin/investors" is active.
 */
export function isNavPathActive(pathname: string, href: string, exact = false) {
  if (pathname === href) return true;
  if (exact) return false;
  return pathname.startsWith(`${href}/`);
}
