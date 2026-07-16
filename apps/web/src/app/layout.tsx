import type { Metadata } from "next";
import "./globals.css";

const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";

export const metadata: Metadata = {
  title: "BSR Hub — Rent. Reuse. Build.",
  description: "A community marketplace for affordable access.",
  icons: { icon: `${basePath}/brand/bsr-icon.png` },
};
export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return <html lang="en"><body>{children}</body></html>;
}
