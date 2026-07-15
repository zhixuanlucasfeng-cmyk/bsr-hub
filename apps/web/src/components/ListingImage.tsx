"use client";

import Image from "next/image";
import { useState } from "react";
import type { Listing } from "../lib/types";

interface ListingImageProps {
  listing: Listing;
  className?: string;
  showType?: boolean;
}

export function ListingImage({ listing, className = "", showType = false }: ListingImageProps) {
  const [failed, setFailed] = useState(false);
  const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";

  return <div className={`listing-image ${className}`.trim()}>
    {failed ? <div className="image-fallback" style={{ background: `linear-gradient(135deg,${listing.accent},#111827)` }}><span>{listing.icon}</span></div> :
      <Image src={`${basePath}${listing.imageSrc}`} alt={listing.imageAlt} fill sizes="(max-width: 650px) 100vw, (max-width: 1000px) 50vw, 25vw" onError={() => setFailed(true)} />}
    {showType && <b className="image-type">{listing.listingType}</b>}
  </div>;
}
