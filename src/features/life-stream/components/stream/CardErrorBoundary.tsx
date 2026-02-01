import type { ReactNode } from "react";
import React, { Component } from "react";

type CardErrorBoundaryProps = {
  children: ReactNode;
  cardId?: string;
};

type CardErrorBoundaryState = {
  hasError: boolean;
  error: Error | null;
};

export class CardErrorBoundary extends Component<
  CardErrorBoundaryProps,
  CardErrorBoundaryState
> {
  state: CardErrorBoundaryState = {
    hasError: false,
    error: null,
  };

  static getDerivedStateFromError(error: Error): CardErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error(`Card ${this.props.cardId ?? "unknown"} crashed:`, error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="life-stream-card life-stream-card--error">
          <div className="life-stream-card__error-content">
            <span className="life-stream-card__error-icon" aria-hidden>
              ⚠️
            </span>
            <span className="life-stream-card__error-text">
              Card failed to render
            </span>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
