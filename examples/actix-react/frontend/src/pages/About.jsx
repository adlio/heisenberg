function About() {
  return (
    <div>
      <h1>About</h1>

      <h2>What is Heisenberg?</h2>
      <p>
        Heisenberg is a Rust library that integrates frontend development
        servers with Rust web frameworks. It proxies frontend dev servers during
        development and serves static assets in production.
      </p>

      <h2>Technology Stack</h2>
      <ul>
        <li>
          <strong>Backend</strong>: actix-web (Rust)
        </li>
        <li>
          <strong>Frontend</strong>: React 18 + React Router DOM + Vite
        </li>
      </ul>

      <h2>How It Works</h2>
      <ol>
        <li>
          In development, Heisenberg proxies requests to the Vite dev server
          with Hot Module Reload (HMR) enabled
        </li>
        <li>In production, Heisenberg serves the built static files</li>
        <li>API requests are handled directly by Actix backend</li>
      </ol>
    </div>
  );
}

export default About;
