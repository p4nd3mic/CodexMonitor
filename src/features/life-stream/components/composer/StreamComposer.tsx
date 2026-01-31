import { useState } from "react";

type StreamComposerProps = {
  onSubmit: (text: string) => void;
};

export function StreamComposer({ onSubmit }: StreamComposerProps) {
  const [value, setValue] = useState("");

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    const trimmed = value.trim();
    if (!trimmed) return;
    onSubmit(trimmed);
    setValue("");
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
        />
        <button className="life-stream-composer__submit" type="submit">
          Add
        </button>
      </form>
    </section>
  );
}
