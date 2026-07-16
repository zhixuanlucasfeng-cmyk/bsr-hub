export interface OptimizedImagePaths {
  small: string;
  large: string;
  detail: string;
}

export function optimizedImagePaths(source: string): OptimizedImagePaths {
  const match = source.match(/^\/images\/(?:listings|categories)\/([^/]+)\.(?:jpe?g|png)$/i);
  if (!match) throw new Error(`Unsupported image source: ${source}`);
  const stem = match[1];
  return {
    small: `/images/optimized/card-sm/${stem}.webp`,
    large: `/images/optimized/card-lg/${stem}.webp`,
    detail: `/images/optimized/detail/${stem}.webp`,
  };
}
