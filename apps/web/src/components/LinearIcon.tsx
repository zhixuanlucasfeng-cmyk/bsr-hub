import type { SVGProps } from "react";

export type IconName = "search" | "shield" | "user" | "truck" | "game" | "camera" | "tool" | "laptop" | "studio" | "reuse" | "support" | "orders" | "chevron" | "sparkle";

const paths: Record<IconName, React.ReactNode> = {
  search: <><circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/></>,
  shield: <><path d="M12 3 5 6v5c0 4.4 2.8 8.3 7 10 4.2-1.7 7-5.6 7-10V6l-7-3Z"/><path d="m9 12 2 2 4-4"/></>,
  user: <><circle cx="12" cy="8" r="4"/><path d="M4 21a8 8 0 0 1 16 0"/></>,
  truck: <><path d="M3 6h11v10H3zM14 10h4l3 3v3h-7z"/><circle cx="7" cy="18" r="2"/><circle cx="18" cy="18" r="2"/></>,
  game: <><path d="M8 8h8a5 5 0 0 1 4.7 6.7l-1 2.8a2 2 0 0 1-3.2.8L14 16h-4l-2.5 2.3a2 2 0 0 1-3.2-.8l-1-2.8A5 5 0 0 1 8 8Z"/><path d="M8 11v4M6 13h4M16.5 12.5h.01M18.5 14.5h.01"/></>,
  camera: <><path d="M4 8h4l2-3h4l2 3h4v11H4z"/><circle cx="12" cy="13" r="4"/></>,
  tool: <><path d="m14 6 4-3 3 3-3 4-4-4ZM13 7 4 16a2 2 0 1 0 3 3l9-9"/></>,
  laptop: <><rect x="4" y="5" width="16" height="11" rx="2"/><path d="M2 19h20"/></>,
  studio: <><path d="M4 21V8l8-5 8 5v13"/><path d="M8 21v-7h8v7M8 10h.01M16 10h.01"/></>,
  reuse: <><path d="m7 7 3-3 3 3"/><path d="M10 4a8 8 0 0 1 8 8M17 17l-3 3-3-3"/><path d="M14 20a8 8 0 0 1-8-8"/></>,
  support: <><path d="M4 12a8 8 0 0 1 16 0v5a2 2 0 0 1-2 2h-2v-7h4M4 12v7H2a2 2 0 0 1-2-2v-5h4Z"/><path d="M16 19c0 2-2 2-4 2"/></>,
  orders: <><rect x="5" y="3" width="14" height="18" rx="2"/><path d="M9 3v3h6V3M9 11h6M9 15h6"/></>,
  chevron: <path d="m9 18 6-6-6-6"/>,
  sparkle: <><path d="m12 3 1.4 4.1L17.5 9l-4.1 1.4L12 14.5l-1.4-4.1L6.5 9l4.1-1.9L12 3Z"/><path d="m19 15 .7 2.3L22 18l-2.3.7L19 21l-.7-2.3L16 18l2.3-.7L19 15Z"/></>,
};

export function LinearIcon({ name, ...props }: SVGProps<SVGSVGElement> & { name: IconName }) {
  return <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" {...props}>{paths[name]}</svg>;
}
