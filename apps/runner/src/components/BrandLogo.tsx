interface BrandLogoProps {
  variant?: "icon" | "horizontal";
  className?: string;
}

const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";

export function BrandLogo({ variant = "icon", className }: BrandLogoProps) {
  const horizontal = variant === "horizontal";

  return (
    <img
      className={className}
      src={`${basePath}/brand/${horizontal ? "bsr-runner-logo.svg" : "bsr-icon.svg"}`}
      alt={horizontal ? "BSR Runner" : "BSR Runner icon"}
      width={horizontal ? 1400 : 512}
      height={horizontal ? 360 : 512}
    />
  );
}
