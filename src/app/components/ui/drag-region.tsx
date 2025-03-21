
interface DragRegionProps {
  className?: string;
}

export function DragRegion({ className = '' }: DragRegionProps) {
  return (
    <div 
      data-tauri-drag-region
      className={`h-8 w-full title-panel ${className}`} 
    />
  );
} 