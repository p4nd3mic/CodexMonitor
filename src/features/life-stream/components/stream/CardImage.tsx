import { useEffect, useMemo, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { CardImage as CardImageData } from "../../types";
import "./StreamCardExtras.css";

export type CardImageProps = {
  image?: CardImageData;
  title?: string;
  emoji?: string;
  size?: "compact" | "expanded";
  className?: string;
  onRequestUpload?: () => void;
};

const STATUS_LABELS: Record<NonNullable<CardImageData>["status"], string> = {
  loading: "Loading image",
  ready: "Image ready",
  missing: "No image yet",
  upload_prompt: "Add a photo",
};

export function CardImage({
  image,
  title,
  emoji = "🖼️",
  size = "compact",
  className,
  onRequestUpload,
}: CardImageProps) {
  if (!image) {
    return null;
  }

  const [imageFailed, setImageFailed] = useState(false);

  useEffect(() => {
    setImageFailed(false);
  }, [image?.url]);

  const status = image?.status ?? "missing";
  const hasImage = Boolean(image?.url) && status === "ready" && !imageFailed;
  const label = STATUS_LABELS[status];
  const showUploadAction = status === "upload_prompt" && Boolean(onRequestUpload);
  const resolvedSrc = useMemo(() => {
    if (!image?.url) return "";
    if (image.url.startsWith("http") || image.url.startsWith("tauri://") || image.url.startsWith("asset://")) {
      return image.url;
    }
    return convertFileSrc(image.url);
  }, [image?.url]);

  return (
    <div
      className={
        `life-stream-card-image is-${size} status-${status}` +
        (className ? ` ${className}` : "")
      }
    >
      {hasImage ? (
        <img
          src={resolvedSrc}
          alt={title ?? "Card image"}
          loading="lazy"
          onError={() => setImageFailed(true)}
        />
      ) : (
        <div className="life-stream-card-image__placeholder">
          {status === "loading" ? (
            <span className="life-stream-card-image__spinner" aria-hidden />
          ) : (
            <span className="life-stream-card-image__emoji" aria-hidden>
              {emoji}
            </span>
          )}
          <span className="life-stream-card-image__label">{label}</span>
          {showUploadAction && (
            <button
              type="button"
              className="life-stream-card-image__action"
              onClick={onRequestUpload}
              disabled={!onRequestUpload}
            >
              Upload photo
            </button>
          )}
        </div>
      )}
    </div>
  );
}
