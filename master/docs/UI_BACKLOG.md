# UI backlog (sweep items that need more than a chrome change)

| Item | Where | Why deferred |
|---|---|---|
| StatusBar job count | `components/StatusBar.svelte` | The hardcoded "0 active jobs" was removed (it was always false). A real count needs a job-count signal in `state.svelte.ts` (new behaviour). |
| Toast wording | `state.svelte.ts` `initJobListener` | "Operation Completed" / "Operation Failed" / "Operation Cancelled" are generic and Title Case; each job should name its result ("Saved x.pdf"). Needs per-op messages, i.e. behaviour in the job layer, not chrome. |
| FAQ load failure | `i18n/faqStore.svelte.ts` | The store swallows load errors and falls back; there is no error signal for an ErrorState. |
| Favourites bar clipped | `components/studio/FavoritesBar.svelte` / Studio layout | At 1200x800 the PINS bar's lower edge sits under the status bar (the canvas box is taller than the visible area). Not caused by the sweep; needs a layout fix in the Studio shell (height of the canvas host). |
| Edit view "Add sample highlight" | `views/EditView.svelte` | Adds a fixed example highlight on page 1; the real highlight tool is in Studio. Candidate to remove or move behind Labs (product decision). |
