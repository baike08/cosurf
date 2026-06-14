import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

// 全局错误边界
class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean; error: string }
> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false, error: '' };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error: error.message + '\n' + error.stack };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('[ErrorBoundary] Caught error:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{ padding: 20, fontFamily: 'monospace', fontSize: 12 }}>
          <h2 style={{ color: 'red' }}>Application Error</h2>
          <pre style={{ whiteSpace: 'pre-wrap', background: '#f5f5f5', padding: 10, borderRadius: 4 }}>
            {this.state.error}
          </pre>
          <button
            onClick={() => window.location.reload()}
            style={{ marginTop: 10, padding: '8px 16px', cursor: 'pointer' }}
          >
            Reload App
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
);
