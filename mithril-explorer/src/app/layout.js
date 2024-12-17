import Image from "next/image";
import Link from "next/link";
import React, { Suspense } from "react";
import { Badge } from "react-bootstrap";
import { Providers } from "@/store/provider";

// These styles apply to every route in the application
import "bootstrap/dist/css/bootstrap.min.css";
import "bootstrap-icons/font/bootstrap-icons.css";
import "./global.css";
import "./mithril-icons.css";

import styles from "./explorer.module.css";

export const metadata = {
  title: "Mithril Explorer",
  description: "Explore a Mithril Network",
};

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <body>
        <link rel="icon" href="/explorer/favicon.svg?v=3" type="image/svg+xml" />

        <Suspense>
          <Providers>
            <div className={styles.container}>
              <main className={styles.main}>
                <h1 className={styles.title}>
                  <Link href="/" className="link-underline-opacity-0 link-body-emphasis ">
                    <Image src="/explorer/logo.png" alt="Mithril Logo" width={55} height={55} />{" "}
                    Mithril Explorer
                  </Link>
                  {process.env.UNSTABLE && (
                    <>
                      {" "}
                      <Badge bg="danger" className="fs-6 align-text-top">
                        Unstable
                      </Badge>
                    </>
                  )}
                </h1>
                {children}
              </main>
            </div>
          </Providers>
        </Suspense>

        <footer className={styles.footer}>
          <span className={styles.logo}>
            <Image src="/explorer/logo.png" alt="Mithril Logo" width={32} height={32} />
          </span>{" "}
          <a href="https://mithril.network/doc">Go back to mithril documentation</a>
        </footer>
      </body>
    </html>
  );
}
