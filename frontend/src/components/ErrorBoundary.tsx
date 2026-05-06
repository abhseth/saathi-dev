import React from "react";

export class ErrorBoundary extends React.Component<
  { children: React.ReactNode; fallback?: React.ReactNode },
  { hasError: boolean; error?: Error }
> {
  state = { hasError: false };
  static getDerivedStateFromError(error: Error) { return { hasError: true, error }; }
  componentDidCatch(error: Error, info: React.ErrorInfo) { console.error(error, info); }
  render() { if (this.state.hasError) return this.props.fallback ?? <div>Error occurred. <button onClick={() => window.location.reload()}>Reload</button></div>; return this.props.children; }
}
