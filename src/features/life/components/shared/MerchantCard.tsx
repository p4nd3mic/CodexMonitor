import type { MerchantStats } from "../../types";

export type MerchantCardProps = MerchantStats;

export function MerchantCard({
  merchantName,
  logoUrl,
  orderCount,
  totalEarnings,
  avgPayout,
  avgMiles,
  avgPerMile,
  tier,
}: MerchantCardProps) {
  return (
    <div className="merchant-card">
      <div className="merchant-card__logo">
        {logoUrl ? (
          <img src={logoUrl} alt={merchantName} loading="lazy" />
        ) : (
          <div className="merchant-card__avatar">
            {getInitials(merchantName)}
          </div>
        )}
      </div>
      <div className="merchant-card__body">
        <div className="merchant-card__title">
          {merchantName}
          {tier ? <span className={`merchant-card__tier tier-${tier}`}>{tier}</span> : null}
        </div>
        <div className="merchant-card__meta">
          {orderCount} orders · ${totalEarnings.toFixed(2)} earned
        </div>
        <div className="merchant-card__meta merchant-card__meta--sub">
          Avg ${avgPayout.toFixed(2)}
          {avgMiles ? ` · ${avgMiles.toFixed(1)} mi` : ""}
          {avgPerMile ? ` · $${avgPerMile.toFixed(2)}/mi` : ""}
        </div>
      </div>
    </div>
  );
}

function getInitials(name: string) {
  return name
    .split(" ")
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}
