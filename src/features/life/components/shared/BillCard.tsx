import type { Bill } from "../../types";

export type BillCardProps = Bill;

export function BillCard({
  name,
  amount,
  frequency,
  nextDueDate,
  autoPay,
  logoUrl,
  category,
}: BillCardProps) {
  const dueSoon = isDueSoon(nextDueDate);
  const daysRemaining = daysUntil(nextDueDate);
  return (
    <div className={`bill-card${dueSoon ? " is-due-soon" : ""}`}>
      <div className="bill-card__logo">
        {logoUrl ? (
          <img src={logoUrl} alt={name} loading="lazy" />
        ) : (
          <div className="bill-card__avatar">{categoryIcon(category)}</div>
        )}
      </div>
      <div className="bill-card__body">
        <div className="bill-card__title">{name}</div>
        <div className="bill-card__meta">
          {formatCurrency(amount)} · {frequency}
          {daysRemaining !== null ? ` · ${daysRemaining}d` : ""}
        </div>
        <div className="bill-card__time">
          Due {formatShortDate(nextDueDate)} {autoPay ? "· Auto-pay" : ""}
        </div>
      </div>
    </div>
  );
}

function formatCurrency(value: number) {
  return value.toLocaleString(undefined, {
    style: "currency",
    currency: "USD",
    maximumFractionDigits: 0,
  });
}

function formatShortDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleDateString(undefined, { month: "short", day: "numeric" });
}

function isDueSoon(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return false;
  const now = new Date();
  const diff = date.getTime() - now.getTime();
  const days = diff / (1000 * 60 * 60 * 24);
  return days >= 0 && days <= 7;
}

function daysUntil(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return null;
  const now = new Date();
  const diff = date.getTime() - now.getTime();
  return Math.max(0, Math.ceil(diff / (1000 * 60 * 60 * 24)));
}

function categoryIcon(category: string) {
  const normalized = category.toLowerCase();
  if (normalized.includes("utility")) return "⚡";
  if (normalized.includes("subscription")) return "📺";
  if (normalized.includes("insurance")) return "🛡️";
  if (normalized.includes("credit")) return "💳";
  if (normalized.includes("rent") || normalized.includes("housing")) return "🏠";
  return "💸";
}
