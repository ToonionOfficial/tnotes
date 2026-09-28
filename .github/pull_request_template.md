## Summary

<!-- What does this change and why? Link issues with `Closes #...`. -->

## Changes

-

## Testing

<!-- Run the groups your change touches. CI runs all of them. -->

Rust (`crates/*`, `apps/server`, `apps/desktop`)

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all-targets --all-features`

Web (`apps/web`)

- [ ] `pnpm --dir apps/web format:check`
- [ ] `pnpm --dir apps/web lint`
- [ ] `pnpm --dir apps/web typecheck`
- [ ] `pnpm --dir apps/web build`

Mobile (`apps/mobile`)

- [ ] `pnpm --dir apps/mobile format:check`
- [ ] `pnpm --dir apps/mobile lint`
- [ ] `pnpm --dir apps/mobile typecheck`
- [ ] `pnpm --dir apps/mobile test`
- [ ] `pnpm --dir apps/mobile run doctor`

<!-- Manual checks (which app or device, what you tapped/typed, what you verified).
For native changes (new native module, config plugin, assets): dev-client rebuild required, Expo Go won't cover it. -->

## Version sync

<!-- `scripts/check-version-sync.py` runs in CI. If this needs a release, bump with `scripts/bump-version.sh` instead of editing versions by hand. -->

- [ ] No version change needed
- [ ] Version bumped (minor for schema or user-facing changes)

## Data migration / sync contract

<!-- None, or describe: new drizzle migration, FTS rebuild, sync envelope or checksum change, and how existing installs and old clients behave. -->

## Breaking changes

<!-- None, or describe the migration path. -->
