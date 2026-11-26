export const M1_MATERIAL_ICON_NAMES = [
  "play_arrow",
  "pause",
  "stop",
  "library_music",
  "volume_up",
] as const;

export type MaterialIconName = (typeof M1_MATERIAL_ICON_NAMES)[number];

export function MaterialIcon({
  name,
  className,
}: {
  name: MaterialIconName;
  className?: string;
}) {
  return (
    <span
      className={
        className
          ? `material-symbols-outlined ${className}`
          : "material-symbols-outlined"
      }
      aria-hidden="true"
    >
      {name}
    </span>
  );
}
