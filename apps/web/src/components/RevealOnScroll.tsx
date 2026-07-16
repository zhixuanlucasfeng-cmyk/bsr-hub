"use client";

import { type ReactNode, useEffect, useRef, useState } from "react";

interface RevealOnScrollProps {
  children: ReactNode;
  delay?: number;
}

export function RevealOnScroll({ children, delay = 0 }: RevealOnScrollProps) {
  const elementRef = useRef<HTMLDivElement>(null);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const element = elementRef.current;
    if (!element) return;

    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      setVisible(true);
      return;
    }

    const observer = new IntersectionObserver(([entry]) => {
      if (!entry.isIntersecting) return;
      setVisible(true);
      observer.disconnect();
    }, { rootMargin: "80px 0px", threshold: 0.08 });

    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  return <div
    ref={elementRef}
    style={{ transitionDelay: `${delay}ms` }}
    className={`transition-[opacity,transform] duration-500 ease-out ${visible ? "translate-y-0 opacity-100" : "translate-y-4 opacity-0"}`}
  >{children}</div>;
}
