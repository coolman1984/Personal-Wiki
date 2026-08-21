import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Personal Wiki Web",
  description: "قاعدة معرفة شخصية للأوامر والأدوات والمصادر الموثوقة.",
  openGraph: {
    title: "Personal Wiki Web",
    description: "اكتشف أوامرك وأمثلتك ومصادرك في مكان واحد.",
    type: "website",
  },
  icons: { icon: "/favicon.svg", shortcut: "/favicon.svg" },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="ar" dir="rtl">
      <body>{children}</body>
    </html>
  );
}
