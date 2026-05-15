## Sensor Pattern Checklist

Before merging any PR that touches sensor fetch, authentication, or data transformation:

- [ ] New sensor behaviour is expressed as TOML spec (`*.sensor.toml`) — not as a Rust module
- [ ] Outbound HTTP calls flow through `host_http_request` (not direct `reqwest` in plugin source)
- [ ] Plugin `allowed_urls` manifest field is populated with the minimum required hostname set
