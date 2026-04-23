import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Fractal",
  description:
    "Fractal is a compiled programming language built in Rust, featuring a powerful compiler and a sleek editor.",
  icons: {
    icon: 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><rect width="100" height="100" rx="20" fill="%230e1117"/><rect x="46" y="25" width="8" height="35" rx="4" fill="%23ff7b72"/><circle cx="50" cy="75" r="6" fill="%23ff7b72"/></svg>',
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
