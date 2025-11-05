import { useState } from "react";

function ApiDemo() {
  const [response, setResponse] = useState(null);
  const [loading, setLoading] = useState(false);
  const [callCount, setCallCount] = useState(0);

  const callAPI = async () => {
    setLoading(true);
    try {
      const res = await fetch("/api/hello");
      const data = await res.json();
      setResponse(data);
      setCallCount((prev) => prev + 1);
    } catch (error) {
      setResponse({ error: error.message });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <h1>API Demo</h1>

      <p>
        This page demonstrates API calls from React to the Actix backend. Click
        the button to make a request to <code>/api/hello</code>.
      </p>

      <div style={{ display: "flex", alignItems: "center", gap: "15px" }}>
        <button onClick={callAPI} disabled={loading}>
          Call API
        </button>
        {callCount > 0 && <span>Requests made: {callCount}</span>}
      </div>

      {response && (
        <div className="response">
          <strong>Response:</strong>
          <pre>{JSON.stringify(response, null, 2)}</pre>
        </div>
      )}

      <h2>Testing</h2>
      <ul>
        <li>Click the button to verify the proxy works</li>
        <li>Edit this file and save to test HMR</li>
        <li>Check the Network tab to see the API request</li>
      </ul>
    </div>
  );
}

export default ApiDemo;
