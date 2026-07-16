import { notFound } from "next/navigation";
import { ListingDetailView } from "../../../components/ListingDetailView";
import { demoCatalog, getDemoListing } from "../../../lib/static-demo";

export function generateStaticParams() {
  return demoCatalog.map((listing) => ({ id:listing.id }));
}

export default async function ListingPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  const listing = getDemoListing(id);
  if (!listing) notFound();
  return <ListingDetailView listing={listing} catalog={demoCatalog}/>;
}
