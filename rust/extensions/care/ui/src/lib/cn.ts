import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/** The shadcn class-composition helper: clsx for conditionals, tailwind-merge
 * to resolve Tailwind conflicts (last utility wins). Every shadcn primitive in
 * this extension imports it. Single responsibility (not a `utils` grab-bag —
 * FILE-LAYOUT). */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
