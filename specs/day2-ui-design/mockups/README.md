<!--
  README.md — Prism Day-2 UI Mockups
  2026-06-25 day-2 side-analysis — UI mockups, do_not_execute, separate from live factory pipeline.
-->

# Prism Day-2 UI Mockups

**Status:** Complete — side-analysis work, do_not_execute.
Separate from the live VSDD factory pipeline. Pending human review and PO/architect adjudication at day-2 morph time.

---

## How to View

Open any `.html` file directly via `file://` in any modern browser (Chrome, Firefox, Safari, Edge).
No build step, no server, no npm required.

Use the **theme toggle** (top-right of every panel) to switch between light and dark.
The toggle sets `document.documentElement.dataset.theme`; default is light.

---

## Files

### Foundation

| File | Description |
|------|-------------|
| `tokens.css` | Shared design token stylesheet — single source of truth for both themes. |
| `style-guide.html` | Full component kit: typography, colors, severity badges, buttons, inputs, tabs, editor chrome, AI callout, app shell, states. |

### S2 — Investigations Console (13 panels)

| File | Screen |
|------|--------|
| `S2-01-triage-queue.html` | Triage Queue |
| `S2-02-case-detail.html` | Case / Incident Detail |
| `S2-03-entity-360.html` | Entity 360 / Observable Profile |
| `S2-04-investigation-workspace.html` | Investigation Workspace |
| `S2-05-results-explorer.html` | Results Explorer (PrismQL editor + coverage banner + OCSF grid) |
| `S2-06-detection-rules.html` | Detection Rule Editor |
| `S2-07-findings.html` | Findings / Alert Queue |
| `S2-08-dashboards.html` | Posture Dashboards |
| `S2-09-reporting.html` | Reporting |
| `S2-10-saved-queries.html` | Saved Queries / Library |
| `S2-11-cache-browser.html` | Cache / Retention Browser |
| `S2-12-sources-connectors.html` | Sources / Connectors |
| `S2-13-satellite-topology.html` | Satellite / Topology |

### U1 — Admin Console (8 panels)

| File | Screen |
|------|--------|
| `U1-01-tenant-management.html` | Tenant Management |
| `U1-02-users-roles.html` | Users & RBAC |
| `U1-03-connector-config.html` | Connector Config |
| `U1-04-credential-rotation.html` | Credential Rotation |
| `U1-05-audit-log.html` | Audit Log |
| `U1-06-health-observability.html` | Health & Observability |
| `U1-07-sso-wizard.html` | SSO Wizard |
| `U1-08-policy-store.html` | Policy Store |

---

## Status: Complete

21 panels x light/dark — all HTML files exist and screenshots are captured.
No panels are pending rollout.

---

## Screenshots

Screenshots were captured by Playwright at 1440px viewport in both themes (44 PNGs total).
They live under `screenshots/`. If absent, the `.html` files are the primary deliverable.

| File | Description |
|------|-------------|
| `screenshots/style-guide-light.png` | Style guide — light |
| `screenshots/style-guide-dark.png` | Style guide — dark |
| `screenshots/triage-queue-light.png` | S2-01 Triage Queue — light |
| `screenshots/triage-queue-dark.png` | S2-01 Triage Queue — dark |
| `screenshots/S2-02-case-detail-light.png` | S2-02 Case Detail — light |
| `screenshots/S2-02-case-detail-dark.png` | S2-02 Case Detail — dark |
| `screenshots/S2-03-entity-360-light.png` | S2-03 Entity 360 — light |
| `screenshots/S2-03-entity-360-dark.png` | S2-03 Entity 360 — dark |
| `screenshots/S2-04-investigation-workspace-light.png` | S2-04 Investigation Workspace — light |
| `screenshots/S2-04-investigation-workspace-dark.png` | S2-04 Investigation Workspace — dark |
| `screenshots/results-explorer-light.png` | S2-05 Results Explorer — light |
| `screenshots/results-explorer-dark.png` | S2-05 Results Explorer — dark |
| `screenshots/S2-06-detection-rules-light.png` | S2-06 Detection Rules — light |
| `screenshots/S2-06-detection-rules-dark.png` | S2-06 Detection Rules — dark |
| `screenshots/S2-07-findings-light.png` | S2-07 Findings — light |
| `screenshots/S2-07-findings-dark.png` | S2-07 Findings — dark |
| `screenshots/S2-08-dashboards-light.png` | S2-08 Dashboards — light |
| `screenshots/S2-08-dashboards-dark.png` | S2-08 Dashboards — dark |
| `screenshots/S2-09-reporting-light.png` | S2-09 Reporting — light |
| `screenshots/S2-09-reporting-dark.png` | S2-09 Reporting — dark |
| `screenshots/S2-10-saved-queries-light.png` | S2-10 Saved Queries — light |
| `screenshots/S2-10-saved-queries-dark.png` | S2-10 Saved Queries — dark |
| `screenshots/S2-11-cache-browser-light.png` | S2-11 Cache Browser — light |
| `screenshots/S2-11-cache-browser-dark.png` | S2-11 Cache Browser — dark |
| `screenshots/S2-12-sources-connectors-light.png` | S2-12 Sources / Connectors — light |
| `screenshots/S2-12-sources-connectors-dark.png` | S2-12 Sources / Connectors — dark |
| `screenshots/S2-13-satellite-topology-light.png` | S2-13 Satellite / Topology — light |
| `screenshots/S2-13-satellite-topology-dark.png` | S2-13 Satellite / Topology — dark |
| `screenshots/U1-01-tenant-management-light.png` | U1-01 Tenant Management — light |
| `screenshots/U1-01-tenant-management-dark.png` | U1-01 Tenant Management — dark |
| `screenshots/U1-02-users-roles-light.png` | U1-02 Users & Roles — light |
| `screenshots/U1-02-users-roles-dark.png` | U1-02 Users & Roles — dark |
| `screenshots/U1-03-connector-config-light.png` | U1-03 Connector Config — light |
| `screenshots/U1-03-connector-config-dark.png` | U1-03 Connector Config — dark |
| `screenshots/U1-04-credential-rotation-light.png` | U1-04 Credential Rotation — light |
| `screenshots/U1-04-credential-rotation-dark.png` | U1-04 Credential Rotation — dark |
| `screenshots/U1-05-audit-log-light.png` | U1-05 Audit Log — light |
| `screenshots/U1-05-audit-log-dark.png` | U1-05 Audit Log — dark |
| `screenshots/U1-06-health-observability-light.png` | U1-06 Health & Observability — light |
| `screenshots/U1-06-health-observability-dark.png` | U1-06 Health & Observability — dark |
| `screenshots/U1-07-sso-wizard-light.png` | U1-07 SSO Wizard — light |
| `screenshots/U1-07-sso-wizard-dark.png` | U1-07 SSO Wizard — dark |
| `screenshots/U1-08-policy-store-light.png` | U1-08 Policy Store — light |
| `screenshots/U1-08-policy-store-dark.png` | U1-08 Policy Store — dark |

---

## Token System

### Architecture

`tokens.css` declares all tokens as CSS custom properties on `:root` / `[data-theme="light"]`
and `[data-theme="dark"]`. Every color, spacing, radius, shadow, transition, and
component-level value is a token. No hardcoded colors appear in the HTML files outside
of `tokens.css`, with the following documented exceptions:

- **SVG chart fills** use literal hex values for cross-browser SVG reliability.
- **Reporting PDF-preview surface** is hardcoded white in both themes to simulate paper.

### Light Mode — 1898 & Co Brand Mapping

| Token | Value | Source |
|-------|-------|--------|
| `--color-accent` | `#ff6a39` | Signature orange — sourced verbatim from 1898andco.burnsmcd.com |
| `--color-accent-hover` | `#cc3b15` | Hover/active darker orange |
| `--color-blue` | `#0057b8` | Brand blue (links / secondary actions) |
| `--color-blue-bright` | `#287ee1` | Bright blue variant |
| `--color-blue-lighter` | `#4099ff` | Lighter blue variant |
| `--color-cyan` | `#71c5e7` | Sky cyan (info / selected tint) |
| `--color-cyan-soft` | `#b8d8eb` | Soft cyan |
| `--color-cyan-mint` | `#a7e5d6` | Mint variant |
| `--color-red` | `#c80f2e` | Brand red (critical / error) |
| `--color-yellow` | `#f6eb62` | Highlight yellow (used very sparingly) |
| `--color-text-primary` | `#393a3c` | Charcoal ink — primary text |
| `--color-text-secondary` | `#63666b` | Secondary / supporting text |

Neutral surfaces: `#ffffff → #f8fafc → #eef0f2 → #ebebeb → #e0e0e0 → #cacaca → #b1b3b3 → #707070 → #4f5257 → #393a3c`.

### Dark Mode Derivation

Dark mode was derived from the same brand hues with the following rules:

1. **Page background:** `#16181c` — deep charcoal, not pure black (avoids harshness, preserves contrast headroom).
2. **Surface stack:** `#16181c → #1f2227 → #2a2e35 → #34383f` — each step is ~+6 L* (perceptual lightness) for visible layer separation.
3. **Orange accent retained:** `#ff6a39` on dark backgrounds provides strong contrast. Large fills (e.g., wide AI callout backgrounds) softened toward `#ff7a4d`.
4. **Blue brightened:** `#4099ff` (vs. `#0057b8` in light) achieves ≥4.5:1 contrast on dark surfaces.
5. **Severity ramp recalibrated:** same hue families (red/orange/yellow/blue/cyan), boosted lightness for dark-bg WCAG AA.
6. **Coverage state ramp recalibrated:** `#4ade80` (answered), `#fcd34d` (degraded), `#ff5a6e` (timeout) — all WCAG AA on dark surfaces.
7. **Text targets:** primary `#eef0f2` ≥ 14:1 on `#16181c`; secondary `#b1b3b3` ≥ 7:1.

Key dark token values:

```
Page bg:          #16181c
Card/panel:       #1f2227
Elevated:         #2a2e35
Borders:          #2a2e35 / #34383f
Text primary:     #eef0f2
Text secondary:   #b1b3b3
Accent orange:    #ff6a39 (same as light)
Brand blue:       #4099ff (brightened from #0057b8)
Sev critical:     #ff5a6e  (bg: #2a0e12)
Sev high:         #ff7a4d  (bg: #2a1509)
Sev medium:       #ffc94d  (bg: #2a2200)
Sev low:          #4099ff  (bg: #0a1930)
Cov answered:     #4ade80
Cov degraded:     #fcd34d
Cov timeout:      #ff5a6e
```

---

## Design Conventions

### Color semantics

- **Orange (`#ff6a39`)** — primary actions (CTA buttons), active nav indicator, critical/high severity badge accents, streaming state indicator. Orange is NOT used on tab underlines.
- **Blue (`#0057b8` light / `#4099ff` dark)** — navigation, links, secondary actions.
- **Purple (`#6d28d9` label, `#f3f0ff` bg in light)** — AI/agent content, machine-labeled data, trust-first indicators. Purple is the exclusive marker for AI-generated content; it is distinct from the brand orange (human actions) and brand blue (navigation).

### Severity and coverage badges

All severity badges and coverage indicators pair color + icon + text for color-blind safety. Color alone is never the sole signal.

### Per-source coverage control

Per-source coverage is rendered as a compact control that expands on demand, showing answered / degraded / timeout state per sensor source.

### PrismQL editor theming

The PrismQL / code editor re-themes with the page (light editor in light mode, dark editor in dark mode). This differs from the VS Code/Monaco convention of always-dark; it was a deliberate decision to maintain visual coherence with the rest of the surface.

### Admin nav placement

Admin is pinned to the bottom of the nav rail with a visual divider, following the conventional pattern for settings/admin sections.

---

## Typography

| Family | Role | License |
|--------|------|---------|
| **Archivo** | Display headers, section titles, uppercase labels, large metrics | Google Fonts — OFL, free |
| **Manrope** | Body text, UI labels, buttons, inputs | Google Fonts — OFL, free |
| **IBM Plex Mono** | PrismQL editor, code cells, mono values | Google Fonts — OFL, free |

All three families load from Google Fonts. No Adobe Fonts kit is required. The previous iteration of these mockups included a fallback stack referencing `acumin-pro-extra-condensed` (Adobe Fonts); that dependency has been replaced by Archivo, which is available without a kit license.

---

## Not Yet Built

The following surfaces are outside the current mockup scope. They are listed here so reviewers can distinguish "not in these mockups" from "omitted by mistake":

- **S3 — Embedded AI Conversation surface** — inline AI analyst pane / chat-over-data interface.
- **S4 — Browser Extension** — lightweight triage / entity-lookup surface.
- **Responsive / mobile breakpoints** — read-only triage at < 768px viewport width.
- **Per-panel state sheets** — dedicated empty-state, error-state, and loading-state variants for each panel (currently handled inline in the panel HTML but not as standalone sheets).

---

## Provenance

Side-analysis capture, 2026-06-25. `do_not_execute`. Does not modify any live factory specs, behavioral contracts, or architectural decision records. Pending PO/architect adjudication before any day-2 morph work begins.
