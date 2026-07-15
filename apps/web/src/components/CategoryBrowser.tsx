"use client";

import Image from "next/image";
import { categories, type CategoryId } from "../lib/categories";

interface CategoryBrowserProps {
  selected: CategoryId;
  onSelect: (categoryId: CategoryId) => void;
}

export function CategoryBrowser({ selected, onSelect }: CategoryBrowserProps) {
  const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";

  return <section className="category-section" aria-labelledby="category-heading">
    <header>
      <div><p className="eyebrow">BROWSE REAL EXAMPLES</p><h2 id="category-heading">What do you need today?</h2></div>
      {selected !== "all" && <button className="category-clear" onClick={() => onSelect("all")}>View all listings</button>}
    </header>
    <div className="category-row">
      {categories.map(category => <button key={category.id} className={`category-card ${selected === category.id ? "selected" : ""}`} aria-pressed={selected === category.id} onClick={() => onSelect(selected === category.id ? "all" : category.id)}>
        <span className="category-photo"><Image src={`${basePath}${category.imageSrc}`} alt="" fill sizes="180px" /></span>
        <span className="category-copy"><strong>{category.label}</strong><small>{category.example}</small></span>
      </button>)}
    </div>
  </section>;
}
