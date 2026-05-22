# Product Reference

This is the durable current-state reference surface for `jk`'s screens, workflows, and presentation
model. Use these docs when you need the product-facing shape of the app rather than implementation
ownership or packet workflow guidance.

Read these files in this order:

1. [`../product-direction.md`](../product-direction.md) for the product scope and bias.
1. [`view-model.md`](view-model.md) for the single-surface presentation model and split-screen
   constraints.
1. [`screens.md`](screens.md) for the stable screen map and how each screen family fits the app.
1. [`workflows.md`](workflows.md) for the current workflow map and where actions belong.

Then use the per-area references as needed:

- [`screens/`](screens/) for screen-specific contracts, current behavior, non-goals, and known gaps.
- [`workflows/`](workflows/) for workflow-specific scope, shipped actions, and CLI-first gaps.

These docs should stand alone as current reference material. They may note non-goals, intentional
gaps, or promotion criteria for behavior that is still CLI-first, but they should not become packet
trackers, phase plans, or progress ledgers.
