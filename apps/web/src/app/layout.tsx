import type { Metadata } from "next";
import { AuthProvider } from "../components/AuthProvider";
import { PerformanceBoot } from "../components/PerformanceBoot";
import "./globals.css";

const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";

export const metadata: Metadata = {
  title: "BSR Hub — Rent. Reuse. Build.",
  description: "A community marketplace for affordable access.",
  icons: { icon: `${basePath}/brand/bsr-icon.png` },
};
export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <head>
        <link
          rel="preload"
          as="image"
          type="image/webp"
          href={`${basePath}/images/optimized/card-sm/ps5-slim.webp`}
          fetchPriority="high"
        />
      </head>
      <body>
        <AuthProvider>
          <PerformanceBoot/>
          {children}
        </AuthProvider>
      </body>
    </html>
  );
}
