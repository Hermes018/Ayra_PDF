# UI backlog (sweep items that need more than a chrome change)

| Item | Where | Why deferred |
|---|---|---|
| StatusBar job count | `components/StatusBar.svelte` | The hardcoded "0 active jobs" was removed (it was always false). A real count needs a job-count signal in `state.svelte.ts` (new behaviour). |
| Toast wording | `state.svelte.ts` `initJobListener` | "Operation Completed" / "Operation Failed" / "Operation Cancelled" are generic and Title Case; each job should name its result ("Saved x.pdf"). Needs per-op messages, i.e. behaviour in the job layer, not chrome. |
| FAQ load failure | `i18n/faqStore.svelte.ts` | The store swallows load errors and falls back; there is no error signal for an ErrorState. |
| PropertyPill | `components/studio/PropertyPill.svelte` | About 25 compact controls in a floating toolbar: colour swatches, stroke and opacity presets, and 10 unlabeled selects with the label in `aria-label`. ui/Select always renders a visible label, so converting needs a compact (hidden-label) Select variant and a swatch Button variant, which is a design decision. Raw px and title-case labels remain. |
