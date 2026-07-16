import { optimizedImagePaths } from "../lib/image-assets";

interface OptimizedImageProps {
  source: string;
  alt: string;
  mode?: "card" | "detail";
  eager?: boolean;
  className?: string;
  sizes?: string;
}

const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";
const withBasePath = (path: string) => `${basePath}${path}`;

export function OptimizedImage({ source, alt, mode = "card", eager = false, className = "", sizes }: OptimizedImageProps) {
  const paths = optimizedImagePaths(source);
  const cardSizes = sizes ?? "(max-width: 640px) 100vw, (max-width: 1280px) 50vw, 25vw";

  return <picture className="contents">
    {mode === "card"
      ? <source type="image/webp" srcSet={`${withBasePath(paths.small)} 480w, ${withBasePath(paths.large)} 960w`} sizes={cardSizes}/>
      : <source type="image/webp" srcSet={withBasePath(paths.detail)}/>
    }
    <img
      src={withBasePath(source)}
      alt={alt}
      width={mode === "card" ? 960 : 1440}
      height={mode === "card" ? 720 : 990}
      loading={eager ? "eager" : "lazy"}
      decoding="async"
      fetchPriority={eager ? "high" : "auto"}
      className={className}
    />
  </picture>;
}
