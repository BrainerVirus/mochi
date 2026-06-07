# Linux Native Window Controls Experiment Results

Environment for all rows unless noted:

- Ubuntu Wayland
- Installed package from unstable release
- Test starts without using the titlebar double-click workaround

| Experiment              | Settings first open | About first open | Update first open | Widget first open | Close/reopen regression | Diagnostics bundle | Result  |
| ----------------------- | ------------------- | ---------------- | ----------------- | ----------------- | ----------------------- | ------------------ | ------- |
| on-demand-visible       | Pass                | Pass             | Pass              | Pass              | Pass                    | Attached           | unstable-20260606.233705 | Proven; promoted |

Final result: `on-demand-visible` fixed native controls for settings and widget on Ubuntu Wayland. The behavior is promoted to the permanent Linux policy. Remaining variants were not needed.
