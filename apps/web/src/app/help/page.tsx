import Link from "next/link";
import { BrandLogo } from "../../components/BrandLogo";
import { SiteFooter } from "../../components/SiteFooter";

const sections = [
  {
    id:"protected-payment",
    eyebrow:"Transaction safety",
    title:"Protected payment",
    copy:"In the classroom demo, BSR shows how a platform could hold payment until the renter or buyer confirms that the transaction is complete. No real card is charged and no money is held on this public website.",
  },
  {
    id:"help-center",
    eyebrow:"Get help",
    title:"Help center",
    copy:"Use the assistant button on the marketplace for rental, listing, workspace, delivery, payment, and deposit guidance. For a team response, open a GitHub issue without posting passwords, payment details, identity documents, or private addresses.",
  },
  {
    id:"terms",
    eyebrow:"Classroom demo",
    title:"Terms of service",
    copy:"BSR Hub is a fictional educational prototype created for a Babson summer program. Listings, identities, orders, prices, delivery tasks, and payments are demonstration data. The prototype is not currently an offer to provide a commercial marketplace service.",
  },
  {
    id:"privacy",
    eyebrow:"Data handling",
    title:"Privacy policy",
    copy:"The public demo does not request real credentials, government identification, exact addresses, or payment information. Demo identity selection and orders are stored only in the current browser session and can be removed by closing the tab or clearing site data.",
  },
];

export default function HelpPage() {
  return <div className="min-h-screen bg-canvas text-ink">
    <header className="border-b border-zinc-200/70 bg-white px-5 py-4 sm:px-8 lg:px-12"><div className="mx-auto flex max-w-[1320px] items-center justify-between"><Link href="/" aria-label="Return to BSR Hub"><BrandLogo variant="horizontal" className="h-11 w-auto"/></Link><Link href="/" className="rounded-full bg-brand px-5 py-3 text-xs font-bold text-white transition hover:bg-brand-deep">Back to marketplace</Link></div></header>
    <main className="px-5 py-16 sm:px-8 lg:px-12 lg:py-24"><div className="mx-auto max-w-[960px]"><p className="text-xs font-extrabold uppercase tracking-[.18em] text-brand">BSR support & policies</p><h1 className="mt-4 max-w-3xl font-sans text-5xl font-bold tracking-[-.04em] text-zinc-950 sm:text-6xl">Clear answers, without dead links.</h1><p className="mt-6 max-w-2xl text-base leading-7 text-zinc-600">These pages describe how the BSR Hub classroom prototype works today and what would need production review before a real launch.</p><nav aria-label="Help page sections" className="mt-9 flex flex-wrap gap-2">{sections.map((section) => <a key={section.id} href={`#${section.id}`} className="rounded-full bg-white px-4 py-2 text-xs font-bold text-zinc-700 shadow-sm ring-1 ring-zinc-200 transition hover:-translate-y-0.5 hover:text-brand">{section.title}</a>)}</nav><div className="mt-14 space-y-6">{sections.map((section) => <section key={section.id} id={section.id} className="scroll-mt-8 rounded-[24px] bg-white p-7 shadow-soft sm:p-10"><p className="text-[10px] font-extrabold uppercase tracking-[.16em] text-brand">{section.eyebrow}</p><h2 className="mt-3 font-sans text-3xl font-bold tracking-tight text-zinc-950">{section.title}</h2><p className="mt-4 text-sm leading-7 text-zinc-600">{section.copy}</p>{section.id === "help-center" && <a href="https://github.com/zhixuanlucasfeng-cmyk/bsr-hub/issues" target="_blank" rel="noreferrer" className="mt-6 inline-flex rounded-full bg-zinc-950 px-5 py-3 text-xs font-bold text-white transition hover:-translate-y-0.5 hover:bg-brand">Contact the BSR team ↗</a>}</section>)}</div></div></main>
    <SiteFooter/>
  </div>;
}
