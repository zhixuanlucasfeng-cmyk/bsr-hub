import type { Metadata } from "next";
import "./globals.css";

const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";

export const metadata: Metadata = {
  title: "BSR Runner — Local help. Flexible earnings.",
  description: "A protected marketplace for neighborhood errands and flexible local work.",
  icons: { icon: `${basePath}/brand/bsr-icon.png` },
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
