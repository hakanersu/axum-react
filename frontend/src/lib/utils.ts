import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * Utility function for merging Tailwind CSS classes.
 *
 * `clsx` handles conditional classes: cn("base", isActive && "active")
 * `twMerge` resolves Tailwind conflicts: cn("px-4", "px-6") → "px-6"
 *
 * Without twMerge, you'd get "px-4 px-6" which is ambiguous (both apply).
 * twMerge understands Tailwind's class structure and keeps only the last one.
 *
 * This is the standard pattern used by shadcn/ui components.
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
