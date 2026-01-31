import { useMemo } from "react";

const formatter = new Intl.DateTimeFormat(undefined, {
  weekday: "short",
  month: "short",
  day: "numeric",
});

type DayPickerProps = {
  currentDate: string;
  onPrevious: () => void;
  onNext: () => void;
  onToday: () => void;
};

export function DayPicker({ currentDate, onPrevious, onNext, onToday }: DayPickerProps) {
  const label = useMemo(() => {
    const date = new Date(`${currentDate}T00:00:00`);
    if (Number.isNaN(date.getTime())) {
      return currentDate;
    }
    return formatter.format(date);
  }, [currentDate]);

  return (
    <section className="life-card life-stream-day-picker">
      <div className="life-stream-day-picker__label">{label}</div>
      <div className="life-segment-control life-stream-day-picker__controls">
        <button type="button" className="life-segment-button" onClick={onPrevious}>
          ◀ Prev
        </button>
        <button type="button" className="life-segment-button" onClick={onToday}>
          Today
        </button>
        <button type="button" className="life-segment-button" onClick={onNext}>
          Next ▶
        </button>
      </div>
    </section>
  );
}
