# Pair ping-pong — Cursor ↔ Codex

One worktree: `/Users/bogdan/kiro/sotto`. One writer. The other reviews.

This is not `forge dispatch pair`. That skill is for same/disagree fleet dispatches. Here the pair is this Cursor session and tmux `til:codex`.

## Roles

**Driver** — has the keyboard. Writes code and tests, runs the slice gates, commits. Does not push unless `PAIR_TURN.md` says so. Does not start the next slice. Stops with: commit hash, gates run, worktree clean, “keyboard released”.

**Navigator** — no product-source edits. Reads `git diff main...HEAD` (or the open PR). Tries to **prove the slice is not soft-launch ready** with file:line or command evidence. If the proof holds, the driver fixes. If it fails, the navigator says **ship**. Cursor owns `gh` (open/merge PRs).

Swap after every **slice** (one PR). Mid-slice swap if the same gate fails twice.

## Rotation (soft launch)

| Slice | Driver | Navigator |
|-------|--------|-----------|
| **K** judge-reliability | Codex | Cursor |
| **L** model-onboarding | Cursor | Codex |
| **M** crash-recovery | Codex | Cursor |
| **N** macos-founder-certification | Cursor | Codex |
| **O** docs-readme-closeout | Codex | Cursor |

Current turn lives in `harness/dispatch/PAIR_TURN.md`. Update it at every handoff.

## Handoff packet

Driver writes (or Cursor pastes to `til:codex`):

1. Slice id (K–O)
2. Branch / commit
3. Gates actually run
4. Known holes you did **not** fix
5. `keyboard released`

Navigator replies with either:

- `SHIP` + why the prove-it-broken pass failed, or
- `FIX` + one defect, file:line, and the expected gate

## Never

- Both writing in this worktree
- Parallel K–O slices
- Rewrite locked EARS
- Push `main` / force-push
- Meeting bot, silent start, cloud STT default
