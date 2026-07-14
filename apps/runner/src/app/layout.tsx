import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "BSR Runner — Local help. Flexible earnings.",
  description: "A protected marketplace for neighborhood errands and flexible local work.",
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
