import { useCallback, useEffect, useRef, useState } from "react";
import "./StreamComposer.css";

declare global {
  interface Window {
    webkitSpeechRecognition: new () => SpeechRecognition;
  }
}

interface SpeechRecognitionEvent extends Event {
  results: SpeechRecognitionResultList;
  resultIndex: number;
}

interface SpeechRecognitionResultList {
  length: number;
  item(index: number): SpeechRecognitionResult;
  [index: number]: SpeechRecognitionResult;
}

interface SpeechRecognitionResult {
  isFinal: boolean;
  length: number;
  item(index: number): SpeechRecognitionAlternative;
  [index: number]: SpeechRecognitionAlternative;
}

interface SpeechRecognitionAlternative {
  transcript: string;
  confidence: number;
}

interface SpeechRecognition extends EventTarget {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  start(): void;
  stop(): void;
  abort(): void;
  onstart: ((this: SpeechRecognition, ev: Event) => void) | null;
  onend: ((this: SpeechRecognition, ev: Event) => void) | null;
  onresult: ((this: SpeechRecognition, ev: SpeechRecognitionEvent) => void) | null;
  onerror: ((this: SpeechRecognition, ev: Event & { error: string }) => void) | null;
}

type StreamComposerProps = {
  onSubmit: (text: string) => Promise<void> | void;
  isSubmitting?: boolean;
  placeholder?: string;
};

export function StreamComposer({
  onSubmit,
  isSubmitting = false,
  placeholder = "Log a meal, delivery, thought, or win...",
}: StreamComposerProps) {
  const [input, setInput] = useState("");
  const [isListening, setIsListening] = useState(false);
  const [speechSupported, setSpeechSupported] = useState(false);
  const recognitionRef = useRef<SpeechRecognition | null>(null);

  useEffect(() => {
    const supported =
      "webkitSpeechRecognition" in window || "SpeechRecognition" in window;
    setSpeechSupported(supported);
  }, []);

  const startListening = useCallback(() => {
    if (!speechSupported || isSubmitting) return;

    try {
      const SpeechRecognitionClass =
        window.webkitSpeechRecognition || (window as any).SpeechRecognition;
      const recognition: SpeechRecognition = new SpeechRecognitionClass();

      recognition.continuous = false;
      recognition.interimResults = true;
      recognition.lang = "en-US";

      recognition.onstart = () => {
        setIsListening(true);
      };

      recognition.onend = () => {
        setIsListening(false);
        recognitionRef.current = null;
      };

      recognition.onresult = (event: SpeechRecognitionEvent) => {
        const results = event.results;
        let transcript = "";

        for (let i = 0; i < results.length; i += 1) {
          transcript += results[i][0].transcript;
        }

        setInput(transcript);

        const lastResult = results[results.length - 1];
        if (lastResult.isFinal && transcript.trim()) {
          setTimeout(() => {
            onSubmit(transcript.trim());
            setInput("");
          }, 300);
        }
      };

      recognition.onerror = (event) => {
        console.error("Speech recognition error:", event.error);
        setIsListening(false);
        recognitionRef.current = null;
      };

      recognitionRef.current = recognition;
      recognition.start();
    } catch (error) {
      console.error("Failed to start speech recognition:", error);
      setIsListening(false);
    }
  }, [isSubmitting, onSubmit, speechSupported]);

  const stopListening = useCallback(() => {
    if (recognitionRef.current) {
      recognitionRef.current.stop();
      recognitionRef.current = null;
    }
    setIsListening(false);
  }, []);

  const toggleListening = useCallback(() => {
    if (isListening) {
      stopListening();
    } else {
      startListening();
    }
  }, [isListening, startListening, stopListening]);

  const handleSubmit = useCallback(
    async (event: React.FormEvent) => {
      event.preventDefault();
      if (isSubmitting) return;
      const trimmed = input.trim();
      if (!trimmed) return;
      await onSubmit(trimmed);
      setInput("");
    },
    [input, isSubmitting, onSubmit],
  );

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      if (event.key === "Enter" && !event.shiftKey) {
        event.preventDefault();
        handleSubmit(event);
      }
    },
    [handleSubmit],
  );

  return (
    <section className="life-card life-stream-composer">
      <div className="life-section-title">Add to your stream</div>
      <form className="life-stream-composer__form" onSubmit={handleSubmit}>
        <input
          className={`life-stream-composer__input${isListening ? " life-stream-composer__input--listening" : ""}`}
          type="text"
          placeholder={isListening ? "Listening..." : placeholder}
          value={input}
          onChange={(event) => setInput(event.target.value)}
          onKeyDown={handleKeyDown}
          disabled={isSubmitting}
          autoComplete="off"
          autoCorrect="off"
          spellCheck={false}
        />
        {speechSupported && (
          <button
            type="button"
            onClick={toggleListening}
            className={`life-stream-composer__mic${isListening ? " life-stream-composer__mic--active" : ""}`}
            disabled={isSubmitting}
            aria-label={isListening ? "Stop listening" : "Start voice input"}
            title={isListening ? "Stop listening" : "Voice input"}
          >
            <span className="life-stream-composer__mic-icon" aria-hidden>
              {isListening ? "🔴" : "🎤"}
            </span>
          </button>
        )}
        <button
          className="life-stream-composer__submit"
          type="submit"
          disabled={isSubmitting || !input.trim()}
          aria-label="Submit"
        >
          {isSubmitting ? (
            <span className="life-stream-composer__spinner" />
          ) : (
            "→"
          )}
        </button>
      </form>
    </section>
  );
}
