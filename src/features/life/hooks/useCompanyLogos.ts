import { useMemo } from "react";
import type { Bill } from "../types";

export function useCompanyLogos(bills: Bill[] = []) {
  return useMemo(() => {
    const map = new Map<string, string>();
    bills.forEach((bill) => {
      if (bill.logoUrl) {
        map.set(bill.name, bill.logoUrl);
      }
    });
    return map;
  }, [bills]);
}
