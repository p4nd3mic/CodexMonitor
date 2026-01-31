export type LifeDomain =
  | "stream"
  | "delivery"
  | "nutrition"
  | "exercise"
  | "media"
  | "youtube"
  | "finance";

export type LifeTimeRange = "today" | "week" | "month" | "lifetime";

export type SortDirection = "asc" | "desc";

export interface LifeDomainConfig {
  id: LifeDomain;
  label: string;
  icon: string;
  accentColor: string;
}

export const LIFE_DOMAINS: LifeDomainConfig[] = [
  { id: "stream", label: "Stream", icon: "🧠", accentColor: "#10b981" },
  { id: "delivery", label: "Delivery", icon: "🚗", accentColor: "#3b82f6" },
  { id: "nutrition", label: "Nutrition", icon: "🍽️", accentColor: "#22c55e" },
  { id: "exercise", label: "Exercise", icon: "🏋️", accentColor: "#22c55e" },
  { id: "media", label: "Media", icon: "🎬", accentColor: "#8b5cf6" },
  { id: "youtube", label: "YouTube", icon: "🎥", accentColor: "#ef4444" },
  { id: "finance", label: "Finance", icon: "💸", accentColor: "#f59e0b" },
];

export interface LifeDomainState {
  timeRange: LifeTimeRange;
  filters: Record<string, string>;
  sortBy: string;
  sortDirection: SortDirection;
  expandedSections: Record<string, boolean>;
}

export interface DashboardMeta {
  domain: LifeDomain | string;
  range: LifeTimeRange | string;
  periodStart: string;
  periodEnd: string;
  generatedAt: string;
  sources: Array<"obsidian" | "supabase">;
  cacheHit?: boolean;
}

// -----------------------------
// Delivery
// -----------------------------

export interface DeliveryStats {
  totalEarnings: number;
  orderCount: number;
  activeHours: number;
  totalMiles?: number;
  hourlyRate: number;
  perMileRate: number;
  avgOrderValue?: number;
  startingAr?: number;
  endingAr?: number;
  whaleCatches?: number;
}

export interface DeliveryOrder {
  id: string;
  startedAt: string;
  merchantName: string;
  logoUrl?: string;
  payout: number;
  miles?: number;
  durationMinutes?: number;
  platform?: "doordash" | "uber" | "grubhub" | "other";
  notes?: string;
  rewardTag?: string;
  warningTag?: string;
}

export interface MerchantStats {
  merchantName: string;
  logoUrl?: string;
  orderCount: number;
  totalEarnings: number;
  avgPayout: number;
  avgMiles?: number;
  avgPerMile?: number;
  tier?: "S" | "A" | "B" | "C" | "D";
}

export interface DeliveryDashboard {
  meta: DashboardMeta;
  stats: DeliveryStats;
  orders: DeliveryOrder[];
  topMerchants: MerchantStats[];
}

// -----------------------------
// Nutrition
// -----------------------------

export interface NutritionStats {
  calories: number;
  protein: number;
  carbs: number;
  fat: number;
  fiber?: number;
  mealCount: number;
}

export interface MealEntry {
  id: string;
  timestamp: string;
  mealType: "breakfast" | "lunch" | "dinner" | "snack";
  description: string;
  foods: string[];
  imageUrl?: string;
  protein?: number;
  carbs?: number;
  fat?: number;
  fiber?: number;
  estimatedCalories?: number;
}

export interface NutritionDashboard {
  meta: DashboardMeta;
  stats: NutritionStats;
  meals: MealEntry[];
  weeklyTrend?: Record<string, number>;
}

export const NUTRITION_GOALS = {
  calories: 2000,
  protein: 180,
  carbs: 150,
  fat: 80,
  fiber: 35,
};

// -----------------------------
// Exercise
// -----------------------------

export interface ExerciseStats {
  workoutCount: number;
  walkingMiles: number;
  activeDays: number;
  currentStreak: number;
}

export interface ExerciseEntry {
  id: string;
  timestamp: string;
  type: "walk" | "strength" | "cardio" | "other";
  description: string;
  miles?: number;
  duration?: number;
}

export interface ExerciseDashboard {
  meta: DashboardMeta;
  stats: ExerciseStats;
  entries: ExerciseEntry[];
  byType: Record<string, number>;
}

// -----------------------------
// Finance
// -----------------------------

export interface Bill {
  id: string;
  name: string;
  logoUrl?: string;
  amount: number;
  dueDay: number;
  frequency: "monthly" | "weekly" | "annual";
  category: string;
  autoPay: boolean;
  nextDueDate: string;
}

export interface FinanceStats {
  monthlyTotal: number;
  dueSoonCount: number;
  autoPayCount: number;
}

export interface FinanceDashboard {
  meta: DashboardMeta;
  stats: FinanceStats;
  bills: Bill[];
  byCategory: Record<string, number>;
  statusMessage?: string;
}

// -----------------------------
// Media
// -----------------------------

export type MediaType = "Film" | "TV" | "Anime" | "Game" | "Book" | "YouTube";
export type MediaStatus = "Completed" | "Backlog";
export type MediaViewMode = "grid" | "list";
export type MediaSortOption = "rating" | "title" | "updated" | "type";

export interface MediaItem {
  id: string;
  title: string;
  type: MediaType;
  status: MediaStatus;
  rating?: number;
  coverUrl?: string;
  createdAt: string;
  updatedAt: string;
  completedAt?: string;
  year?: number;
  notes?: string;
}

export interface MediaLibrary {
  meta: DashboardMeta;
  totalCount: number;
  completedCount: number;
  backlogCount: number;
  avgRating: number;
  items: MediaItem[];
}

export interface MediaFilterState {
  type: MediaType | "all";
  status: MediaStatus | "all";
  search: string;
  sort: MediaSortOption;
  viewMode: MediaViewMode;
}

export interface MediaCoverSummary {
  total: number;
  found: number;
  skipped: number;
  failed: number;
}

export interface FixCoversResult {
  fixed: number;
  already_valid: number;
  failed: string[];
}

export interface CoverRebuildResult {
  total: number;
  rebuilt: number;
  failed: string[];
}

export interface CoverOverride {
  mediaId: string;
  coverUrl: string;
  source: "manual";
}

// -----------------------------
// YouTube
// -----------------------------

export type YouTubeTier = "S" | "A" | "B" | "C";
export type YouTubeStage =
  | "idea"
  | "notes"
  | "outline"
  | "draft"
  | "script"
  | "ready"
  | "published";

export interface YouTubeIdea {
  id: string;
  title: string;
  slug?: string;
  tier: YouTubeTier;
  stage: YouTubeStage;
  createdAt: string;
  updatedAt: string;
}

export interface YouTubeLibrary {
  meta: DashboardMeta;
  totalCount: number;
  inProgressCount: number;
  publishedCount: number;
  items: YouTubeIdea[];
}

export interface YouTubeFilterState {
  tier: YouTubeTier | "all";
  stage: YouTubeStage | "all";
  search: string;
  sort: "tier" | "stage" | "title" | "updated";
  viewMode: "grid" | "list";
}
