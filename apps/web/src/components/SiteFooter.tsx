import Link from "next/link";
import { footerGroups, type FooterLink } from "../lib/footer-links";
import { BrandLogo } from "./BrandLogo";

const linkClass = "group inline-flex items-center gap-1.5 rounded-sm text-xs text-white/58 transition hover:text-white focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-accent";

function FooterDestination({ link }: { link: FooterLink }) {
  if (link.external) return <a href={link.href} target="_blank" rel="noreferrer" className={linkClass}>{link.label}<span aria-hidden="true" className="text-[10px] opacity-55 transition group-hover:-translate-y-0.5 group-hover:translate-x-0.5">↗</span></a>;
  return <Link href={link.href} className={linkClass}>{link.label}<span aria-hidden="true" className="text-[10px] opacity-0 transition group-hover:translate-x-0.5 group-hover:opacity-70">→</span></Link>;
}

export function SiteFooter() {
  return <footer className="bg-[#111018] px-5 py-14 text-white sm:px-8 lg:px-12"><div className="mx-auto grid max-w-[1320px] gap-10 md:grid-cols-[1.4fr_repeat(3,1fr)]"><div><BrandLogo variant="horizontal" className="footer-logo h-auto w-44 rounded-lg"/><p className="mt-5 max-w-sm text-sm leading-6 text-white/55">A community marketplace for affordable access to products, creative spaces, and useful second-hand goods.</p></div>{footerGroups.map((group) => <nav key={group.title} aria-label={`${group.title} footer links`} className="footer-link-nav"><h3 className="text-sm font-bold">{group.title}</h3><div className="mt-4 flex flex-col items-start gap-3">{group.links.map((link) => <FooterDestination key={link.label} link={link}/>)}</div></nav>)}</div><div className="mx-auto mt-12 flex max-w-[1320px] flex-col gap-2 border-t border-white/10 pt-6 text-[10px] text-white/38 sm:flex-row sm:justify-between"><p>© 2026 BSR Hub classroom venture.</p><p>Demo marketplace · Fictional users · No real payments or private addresses</p></div></footer>;
}
