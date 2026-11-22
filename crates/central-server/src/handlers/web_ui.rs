use poem::{IntoResponse, Route, endpoint::StaticFilesEndpoint, get, handler, web::Html};
use std::path::Path;

const CENTRAL_SERVER_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>X-Road® Central Server — Admin Portal</title>
  <style>
    :root {
      --primary: #004b87;
      --primary-dark: #003366;
      --secondary: #009688;
      --bg: #f5f7fb;
      --surface: #ffffff;
      --text: #212529;
      --text-muted: #6c757d;
      --border: #dee2e6;
      --success: #28a745;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; background: var(--bg); color: var(--text); }
    header { background: var(--primary); color: white; padding: 1rem 2rem; display: flex; align-items: center; justify-content: space-between; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
    .brand { display: flex; align-items: center; gap: 12px; font-size: 1.3rem; font-weight: 700; }
    .brand-badge { background: var(--secondary); padding: 4px 8px; border-radius: 4px; font-size: 0.75rem; text-transform: uppercase; font-weight: 600; }
    nav { display: flex; gap: 1rem; }
    nav a { color: white; text-decoration: none; font-size: 0.9rem; padding: 6px 12px; border-radius: 4px; transition: background 0.2s; }
    nav a:hover, nav a.active { background: rgba(255,255,255,0.15); }
    main { max-width: 1200px; margin: 2rem auto; padding: 0 1rem; }
    .hero { background: var(--surface); border-radius: 8px; padding: 2rem; margin-bottom: 2rem; border: 1px solid var(--border); box-shadow: 0 1px 3px rgba(0,0,0,0.05); }
    .hero h1 { font-size: 1.8rem; margin-bottom: 0.5rem; color: var(--primary-dark); }
    .hero p { color: var(--text-muted); font-size: 1rem; margin-bottom: 1.5rem; }
    .badge-bar { display: flex; gap: 12px; flex-wrap: wrap; }
    .stat-badge { background: #e8f4fd; color: var(--primary); border: 1px solid #bce0fd; border-radius: 20px; padding: 6px 14px; font-size: 0.85rem; font-weight: 600; display: flex; align-items: center; gap: 6px; }
    .stat-badge.ok { background: #eafaf1; color: var(--success); border-color: #a3e9c0; }
    .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); gap: 1.5rem; }
    .card { background: var(--surface); border-radius: 8px; border: 1px solid var(--border); padding: 1.5rem; transition: transform 0.15s, box-shadow 0.15s; }
    .card:hover { transform: translateY(-2px); box-shadow: 0 4px 8px rgba(0,0,0,0.08); }
    .card h3 { font-size: 1.15rem; margin-bottom: 0.5rem; color: var(--primary-dark); display: flex; justify-content: space-between; align-items: center; }
    .card p { color: var(--text-muted); font-size: 0.9rem; margin-bottom: 1.2rem; line-height: 1.4; }
    .card a.btn { display: inline-block; background: var(--primary); color: white; text-decoration: none; padding: 8px 16px; border-radius: 4px; font-size: 0.85rem; font-weight: 600; }
    .card a.btn:hover { background: var(--primary-dark); }
    footer { text-align: center; margin: 3rem 0 2rem; color: var(--text-muted); font-size: 0.85rem; }
  </style>
</head>
<body>
  <header>
    <div class="brand">
      <span>X-Road® Central Server</span>
      <span class="brand-badge">Rust Engine 7.x</span>
    </div>
    <nav>
      <a href="/" class="active">Overview</a>
      <a href="/swagger-ui" target="_blank">Swagger OpenAPI</a>
      <a href="/internalconf/shared-params.xml" target="_blank">Global Configuration XML</a>
    </nav>
  </header>
  <main>
    <div class="hero">
      <h1>Central Server Governance Dashboard</h1>
      <p>High-performance Rust implementation of the X-Road 7.x Central Server. Governing member registries, approved certification and timestamping authorities, global security groups, and shared parameters distribution.</p>
      <div class="badge-bar">
        <div class="stat-badge ok">● Central Server Engine: Operational</div>
        <div class="stat-badge">Instance: DEV</div>
        <div class="stat-badge">OpenTelemetry & Jaeger: Active</div>
        <div class="stat-badge">Admin API: OpenAPI 3.0</div>
      </div>
    </div>
    <div class="grid">
      <div class="card">
        <h3>Members & Subsystems <span>👥</span></h3>
        <p>Register and manage trust participants, member classes, and federated subsystem identifiers.</p>
        <a href="/swagger-ui#/members" class="btn">Manage Members</a>
      </div>
      <div class="card">
        <h3>Security Servers <span>🛡️</span></h3>
        <p>Register security server instances, map authentication certificates, and authorize proxy clients.</p>
        <a href="/swagger-ui#/security-servers" class="btn">View Security Servers</a>
      </div>
      <div class="card">
        <h3>Global Groups <span>🌐</span></h3>
        <p>Define governance security groups accessible across security servers for uniform access control.</p>
        <a href="/swagger-ui#/global-groups" class="btn">Manage Groups</a>
      </div>
      <div class="card">
        <h3>Certification Services (CA) <span>📜</span></h3>
        <p>Manage approved trust service providers, intermediate CA certificates, and OCSP responders.</p>
        <a href="/swagger-ui#/certification-services" class="btn">Manage CAs</a>
      </div>
      <div class="card">
        <h3>Timestamping Services (TSA) <span>⏱️</span></h3>
        <p>Configure RFC 3161 compliant time-stamping authorities for non-repudiation message signing.</p>
        <a href="/swagger-ui#/timestamping-services" class="btn">Manage TSAs</a>
      </div>
      <div class="card">
        <h3>Shared Parameters Distribution <span>📦</span></h3>
        <p>Generate and distribute the canonical signed global configuration shared-params.xml payload.</p>
        <a href="/internalconf/shared-params.xml" target="_blank" class="btn">Download Shared Params</a>
      </div>
    </div>
  </main>
  <footer>
    X-Road® Central Server — Pure Rust Architecture • Conforming to X-Road 7.x Standard Specifications
  </footer>
</body>
</html>
"#;

#[handler]
async fn handle_embedded_ui() -> impl IntoResponse {
    Html(CENTRAL_SERVER_UI_HTML)
}

/// Creates a route serving either static Vue.js assets (if built) or the embedded Admin UI dashboard.
pub fn central_server_web_ui() -> Route {
    let custom_path = std::env::var("XROAD_UI_PATH").ok();
    let default_paths = ["java/src/central-server/admin-service/ui/dist", "ui/dist"];

    let static_dir = custom_path.or_else(|| {
        default_paths
            .iter()
            .find(|p| Path::new(p).exists())
            .map(|s| s.to_string())
    });

    if let Some(dir) = static_dir {
        Route::new()
            .nest("/", StaticFilesEndpoint::new(dir).index_file("index.html"))
            .at("/ui", get(handle_embedded_ui))
    } else {
        Route::new()
            .at("/", get(handle_embedded_ui))
            .at("/ui", get(handle_embedded_ui))
    }
}
