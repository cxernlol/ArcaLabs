# SPEC: Smooth workflow camera on scroll

## 1. Goal & User Story

Visitors scrolling through the marketing site's workflow should see the diagram camera follow the workflow continuously instead of jumping between focused nodes. The existing focus sequence and reduced-motion behavior remain unchanged.

## 2. Inputs & Outputs

**Inputs**

- Normalized workflow-section scroll progress from `0` to `1`
- Existing sequence of workflow focus nodes
- User `prefers-reduced-motion` setting

**Outputs**

- Diagram camera interpolates between the overview and existing focus-node positions as scroll progress changes.
- Existing node focus highlighting advances through the same sequence.
- Network calls: existing Vercel Web Analytics page-view requests on the hosted site; this change adds none.

## 3. Acceptance Criteria

- [ ] Scrolling through the workflow moves the camera between focus positions without snapping directly to a node at a step boundary.
- [ ] Focus highlighting follows the existing node sequence and the camera returns to the overview when the workflow section ends.
- [ ] Progress is clamped to `0`–`1`; the current progress is applied after the embedded diagram loads.
- [ ] With reduced motion enabled, the diagram remains static and usable.
- [ ] `pnpm --dir apps/web build` succeeds, and browser scrolling through the workflow produces no console errors.

## 4. Edge Cases & Error States

| Situation | Behavior |
|---|---|
| Embedded diagram loads after scrolling begins | Apply the current scroll progress when the iframe loads. |
| Progress is outside `0`–`1` | Clamp it to the nearest endpoint. |
| Reduced motion is enabled | Do not move the camera in response to scrolling; keep the diagram usable. |

## 5. Out of Scope

- Changing workflow nodes, labels, or their order
- Changing the sticky section layout or duration
- Adding animation controls, analytics, network calls, or dependencies
