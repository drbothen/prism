<!--
  README.md — Prism Day-2 UI Mockups
  2026-06-25 day-2 side-analysis — UI mockups, do_not_execute, separate from live factory pipeline.
-->

# Prism Day-2 UI Mockups

**Status:** Proposed/Draft — Side-analysis work, do_not_execute.
Separate from the live VSDD factory pipeline. Pending human review and PO/architect adjudication at day-2 morph time.

---

## Files

| File | Description |
|------|-------------|
| `tokens.css` | Shared design token stylesheet. Single source of truth for both themes. |
| `style-guide.html` | Full component kit: typography, colors, severity badges, buttons, inputs, tabs, editor chrome, AI callout, app shell, states. |
| `S2-01-triage-queue.html` | Flagship panel 1 — Triage Queue (Screen 1). Full app shell, real seeded data, theme toggle. |
| `S2-05-results-explorer.html` | Flagship panel 2 — Results Explorer (Screen 5). PrismQL editor + coverage banner + OCSF grid + entity mode. |
| `screenshots/` | Playwright-captured PNGs (see below). |

---

## How to Open

Open any `.html` file directly via `file://` in any modern browser (Chrome, Firefox, Safari, Edge).
No build step, no server, no npm required.

Use the **☀️ / 🌙 toggle** (top-right) to switch between light and dark modes.
The theme is persisted in `localStorage` so it survives page refresh.

---

## Token System

### Architecture

`tokens.css` declares all tokens as CSS custom properties on `:root` / `[data-theme="light"]`
(which produce the same values) and `[data-theme="dark"]`.

Every color, spacing, radius, shadow, transition, and component-level value is a token.
No hardcoded colors appear in the HTML files outside of `tokens.css`.

### Light Mode → 1898 & Co Brand Mapping

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

Final dark token values (key entries):

```
Page bg:          #16181c
Card/panel:       #1f2227
Elevated:         #2a2e35
Borders:          #2a2e35 / #34383f
Text primary:     #eef0f2
Text secondary:   #b1b3b3
Accent orange:    #ff6a39 (same)
Brand blue:       #4099ff (brightened)
Sev critical:     #ff5a6e  (bg: #2a0e12)
Sev high:         #ff7a4d  (bg: #2a1509)
Sev medium:       #ffc94d  (bg: #2a2200)
Sev low:          #4099ff  (bg: #0a1930)
Cov answered:     #4ade80
Cov degraded:     #fcd34d
Cov timeout:      #ff5a6e
```

---

## Font Licensing

| Family | Usage | License |
|--------|-------|---------|
| **Manrope** | Body text, UI labels, buttons, inputs | Google Fonts — OFL, free |
| **acumin-pro-extra-condensed** | Display headers (section titles, large metrics) | **Adobe Fonts — requires active Adobe Creative Cloud or Adobe Fonts kit license**. The mockups use a fallback stack (`'Archivo', 'Saira Condensed', sans-serif`) so they render without the kit. |
| **IBM Plex Mono** | PrismQL editor, code/mono cells | Google Fonts — OFL, free |

For production deployment, obtain the Adobe kit license and embed the `acumin-pro-extra-condensed` kit URL in the `<head>`.

---

## Screenshots

Screenshots were captured by Playwright at 1440×900px viewport in both themes.

| File | Description |
|------|-------------|
| `screenshots/style-guide-light.png` | Style guide — light mode |
| `screenshots/style-guide-dark.png` | Style guide — dark mode |
| `screenshots/triage-queue-light.png` | Triage Queue — light mode |
| `screenshots/triage-queue-dark.png` | Triage Queue — dark mode |
| `screenshots/results-explorer-light.png` | Results Explorer — light mode |
| `screenshots/results-explorer-dark.png` | Results Explorer — dark mode |

If screenshots are absent, open the HTML files directly in a browser — they are the primary deliverable.

---

## Remaining Panels (Pass 2)

The following ~19 panels are ready to roll out once this visual foundation is approved:

| # | Screen | Notes |
|---|--------|-------|
| SCR-02 | Case / Incident Detail | Tabs: Overview, Evidence, Entities, Timeline, Actions, AI Insights |
| SCR-03 | Entity 360 / Observable Profile | Three-pane; Cytoscape relationship graph |
| SCR-04 | Investigation Workspace | Monaco editor; NL toggle; time/source selectors |
| SCR-06 | Detection Rule Editor | Split-pane; lifecycle stepper; backtest panel |
| SCR-07 | Findings / Alert Queue | Source coverage per finding; replay link |
| SCR-08 | Posture Dashboards | ECharts: trend, pie, MITRE heatmap |
| SCR-09 | Reporting | Report builder; PDF export |
| SCR-10 | Saved Queries / Library | Recipes tab; parameterized queries |
| SCR-11 | Cache / Retention Browser | Hot/cold tier; policy source transparency |
| SCR-12 | Sources / Connectors | Connector health; configure-schema wizard |
| SCR-13 | Satellite / Topology | Tree view; degraded-subtree propagation |
| U1-01 | Admin: Tenant Management | Multi-tenant; per-tenant DEK summary |
| U1-02 | Admin: Users & RBAC | Role assignment; custom roles |
| U1-03 | Admin: Credentials | Write-only/masked; rotation workflow |
| U1-04 | Admin: Audit Log | Immutable log; search/filter |
| U1-05 | Admin: SSO Wizard | SAML/OIDC; test-connection flow |
| U1-06 | Admin: Platform Health | Connector up/down; ingestion metrics |
| U1-07 | Admin: Connector Config | Dynamic connector schema wizard |
| Mobile | Responsive adaptations | Read-only triage on SM (<768px) |

---

## Design Decisions for Human Confirmation

Before rolling out the remaining panels, please confirm:

1. **Accent sparsity:** Orange `#ff6a39` currently appears only on: primary CTA buttons, active nav indicator dot, severity badge left-border (critical/high), new-case banner, and streaming state indicator. Is this sparsity level correct, or should the brand orange appear in more places (e.g., active tab underline)?

2. **Coverage banner prominence:** The per-source coverage banner is rendered as a full-width colored band above the results grid. This is intentional — it is prism's signature differentiator and should be impossible to miss. Confirm this level of visual weight is right, or should it be more subtle (collapsed by default, expandable)?

3. **Editor theme:** The PrismQL editor uses an always-dark catppuccin-style palette regardless of the page theme. This follows the VS Code / Monaco convention. Confirm this is the right call, or should the editor also re-theme to light?

4. **AI callout color:** AI-generated content uses a purple/violet accent (`#6d28d9` label, `#f3f0ff` bg in light). This distinguishes AI from the brand orange (human actions) and brand blue (navigation). Confirm this distinction is intentional — or prefer a different color for AI content.

5. **Typography display font:** Without an Adobe Fonts kit, `Archivo` renders for all display/uppercase headings. The final product needs the Adobe kit for `acumin-pro-extra-condensed`. Confirm the kit will be available, or select an alternative free condensed font (options: Barlow Condensed, Saira Condensed, Bebas Neue).

6. **Left nav grouping:** Nav currently uses labeled groups (Investigations / Query / Detections / Dashboards / Data / Admin). Should Admin be separated at the bottom of the nav rail with a visual divider, as is conventional for settings/admin sections?
