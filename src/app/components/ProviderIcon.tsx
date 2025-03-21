import Image from 'next/image';

export interface ProviderIconProps {
  provider: string;
  url?: string;
  width?: number;
  height?: number;
}

export function ProviderIcon({ provider, url, width = 40, height = 40 }: ProviderIconProps) {
  // Convert provider name to display name (e.g., "google-drive" -> "Google Drive")
  const displayName = provider
    .split('-')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');

  return (
    <Image 
      src={url ? url : `/images/tools/${provider}.svg`}
      width={width}
      height={height}
      alt={displayName}
      className="provider-icon"
    />
  );
}
