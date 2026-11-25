# ADR-012: Cargo and pnpm monorepo

**Status:** Proposed<br>
**Owner:** SAYT project owner<br>
**Date:** 2026-11-25<br>
**Version:** 1

## Decision

Keep the desktop shell and Rust libraries in one repository with a Cargo workspace and a pnpm workspace.

## Context

The desktop UI calls into a Rust process and will share versioned contracts. A single contributor needs one change, lockfile review, and CI run to keep those boundaries aligned.

## Options

- One Cargo + pnpm monorepo: atomic cross-language changes and unified CI; workspace CI grows with platform and feature count.
- Separate Rust and UI repositories: independent release cadence; contract and compatibility changes become cross-repository coordination work.

## Evidence

This follows the founding charter. M0 CI is configured to cover all three desktop operating systems; the workflow has not yet run.

## Consequences

Enables atomic changes to frontend, shell, and engine contracts. Requires ownership of both toolchains and the cross-platform CI matrix.

## Revisit when

The project gains independent maintainers or a component needs a separate release/security boundary.
