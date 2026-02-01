import { useState } from "react";

type StreamComposerProps = {
  onSubmit: (text: string) => Promise<void> | void;
};

export function StreamComposer({ onSubmit }: StreamComposerProps) {
  const [value, setValue] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (isSubmitting) return;
    const trimmed = value.trim();
    if (!trimmed) return;
    setIsSubmitting(true);
    try {
      await onSubmit(trimmed);
      setValue("");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <section className="life-card life-stream-composer">
      <div className="life-section-title">Add to your stream</div>
      <form className="life-stream-composer__form" onSubmit={handleSubmit}>
        <input
          className="life-stream-composer__input"
          type="text"
          placeholder="Log a meal, delivery, thought, or win..."
          value={value}
          onChange={(event) => setValue(event.target.value)}
          disabled={isSubmitting}
        />
        <button className="life-stream-composer__submit" type="submit" disabled={isSubmitting}>
          {isSubmitting ? "Sending..." : "Add"}
        </button>
      </form>
    </section>
  );
}
