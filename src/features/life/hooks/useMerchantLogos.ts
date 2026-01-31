import { useMemo } from "react";
import type { DeliveryOrder, MerchantStats } from "../types";

export function useMerchantLogos(
  merchants: MerchantStats[] = [],
  orders: DeliveryOrder[] = [],
) {
  return useMemo(() => {
    const map = new Map<string, string>();
    merchants.forEach((merchant) => {
      if (merchant.logoUrl) {
        map.set(merchant.merchantName, merchant.logoUrl);
      }
    });
    orders.forEach((order) => {
      if (order.logoUrl && !map.has(order.merchantName)) {
        map.set(order.merchantName, order.logoUrl);
      }
    });
    return map;
  }, [merchants, orders]);
}
