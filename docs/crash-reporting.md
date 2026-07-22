# Crash reporting contract

Crash reporting is optional, local-first and disabled by default. The app works
fully offline and always keeps its local log independently of telemetry.

## Rules

1. The Sentry-compatible client is initialized only after explicit opt-in.
2. `GLITCHTIP_DSN` is supplied by a protected build environment and is never
   committed. Builds without it are normal and keep reporting disabled.
3. `before_send` removes host identity, client media names and filesystem paths.
4. Media, metadata and report contents are never attached to an event.
5. A network or reporting failure cannot affect analysis or offload verdicts.

The implementation is in `src-tauri/src/crash.rs`. Unit tests cover opt-in and
path scrubbing. The base client does not provide durable offline event delivery;
the local application log remains the source of truth when no network exists.

## Verification

- Build once without `GLITCHTIP_DSN`: no reporting client is enabled.
- Build in a protected test environment with a non-production DSN, opt in and
  send a synthetic event.
- Confirm that paths, filenames, host and user information are absent.
- Opt out and confirm that remote events stop while local logs continue.
- Disconnect the network and confirm the application/offload is unaffected.

Private endpoint names, signing credentials and infrastructure runbooks belong
in the operator's private infrastructure repository, not this project.
