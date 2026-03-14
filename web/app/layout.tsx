import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'Fractal — A Language Built in Rust',
  description: 'Fractal is a compiled programming language built in Rust, featuring a powerful compiler and a sleek editor.',
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
