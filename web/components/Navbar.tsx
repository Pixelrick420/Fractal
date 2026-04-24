"use client";
import { useEffect, useState } from "react";
import Link from "next/link";
import styles from "./navbar.module.css";

interface NavbarProps {
  sidebar?: boolean;
}

export default function Navbar({ sidebar = false }: NavbarProps) {
  const [isMobile, setIsMobile] = useState(false);

  useEffect(() => {
    const check = () => setIsMobile(window.innerWidth < 600);
    check();
    window.addEventListener("resize", check);
    return () => window.removeEventListener("resize", check);
  }, []);

  const navClass = sidebar ? styles.sidebarNav : styles.nav;
  const linksClass = sidebar ? styles.sidebarLinks : styles.links;

  return (
    <nav className={navClass}>
      <Link href="/" className={styles.logo}>
        <span className={styles.logoKw}>!</span>fractal
      </Link>
      {!isMobile && (
        <div className={linksClass}>
          <Link href="/demo">Demo</Link>
          <Link href="/docs">Docs</Link>
          <Link href="/#download" className={styles.cta}>
            Download
          </Link>
        </div>
      )}
    </nav>
  );
}
