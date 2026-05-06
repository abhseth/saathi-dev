import React from "react";

type SkeletonBlockProps = {
  variant?: "block" | "text" | "circle";
  width?: string;
};

export function SkeletonBlock({ variant = "block", width }: SkeletonBlockProps) {
  return (
    <div
      className={`skeleton-${variant}`}
      style={width ? { width } : undefined}
    />
  );
}
