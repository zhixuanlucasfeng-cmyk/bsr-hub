export interface FooterLink {
  label: string;
  href: string;
  external?: boolean;
}

export interface FooterGroup {
  title: string;
  links: FooterLink[];
}

export const footerGroups: FooterGroup[] = [
  {
    title: "Marketplace",
    links: [
      { label:"Explore nearby", href:"/#market" },
      { label:"Rent products", href:"/?type=rental#market" },
      { label:"Book spaces", href:"/?type=workspace#market" },
      { label:"Second-hand", href:"/?type=sale#market" },
    ],
  },
  {
    title: "Trust & help",
    links: [
      { label:"Protected payment", href:"/help/#protected-payment" },
      { label:"Help center", href:"/help/" },
      { label:"Terms of service", href:"/help/#terms" },
      { label:"Privacy policy", href:"/help/#privacy" },
    ],
  },
  {
    title: "Impact",
    links: [
      { label:"UN SDG 8 — Decent work", href:"https://sdgs.un.org/goals/goal8", external:true },
      { label:"UN SDG 10 — Reduced inequalities", href:"https://sdgs.un.org/goals/goal10", external:true },
      { label:"Babson Summer Study", href:"https://www.babson.edu/summer-at-babson/high-school-learners/summer-study/", external:true },
      { label:"Contact the BSR team", href:"https://github.com/zhixuanlucasfeng-cmyk/bsr-hub/issues", external:true },
      { label:"BSR Hub on GitHub", href:"https://github.com/zhixuanlucasfeng-cmyk/bsr-hub", external:true },
    ],
  },
];
