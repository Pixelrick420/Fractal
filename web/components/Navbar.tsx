"use client";
import Link from "next/link";
import styles from "./navbar.module.css";

interface NavbarProps {
  sidebar?: boolean;
}

export default function Navbar({ sidebar = false }: NavbarProps) {
  const navClass = sidebar ? styles.sidebarNav : styles.nav;
  const linksClass = sidebar ? styles.sidebarLinks : styles.links;
  
  return (
    <nav className={navClass}>
      <Link href="/" className={styles.logo}>
        <span className={styles.logoKw}>!</span>fractal
      </Link>
      <div className={linksClass}>
        <Link href="/demo">Demo</Link>
        <Link href="/docs">Docs</Link>
        <Link href="/#download" className={styles.cta}>
          Download
        </Link>
      </div>
    </nav>
  );
}