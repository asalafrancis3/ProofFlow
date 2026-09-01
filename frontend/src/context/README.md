# State Management Convention

This project has no separate global store (no Redux/Zustand/Jotai). State is split
across two layers, chosen by what the state represents:

## Server-cache state → `@tanstack/react-query`

Anything fetched from the Soroban contract or backend (users, jobs, milestones,
escrow, disputes, reputation, activity) lives in React Query, via the
hooks in `frontend/src/hooks/`. React Query already owns caching, refetching,
and invalidation for this data — it should never be copied into a Context.

## Client-only global state → `frontend/src/context/`

Cross-cutting state that isn't fetched from a server and needs to be readable
from anywhere in the tree lives here:

- **`WalletContext`** — the connected Freighter wallet address. This is the
  single source of truth for wallet identity.
- **`AuthContext`** — the signed-in user's profile (`role`, `name`). It derives
  the user's `address` from `WalletContext` rather than storing its own copy,
  so wallet identity and auth identity can never drift apart. `AuthProvider`
  must be mounted inside `WalletProvider` (see `main.tsx`) because it calls
  `useWallet()` internally. `logout()` clears the profile *and* disconnects
  the wallet, so both states always change together.
- **`ContractContext`** — the active Soroban network/contract config.
- **`ThemeProvider`** — light/dark theme, delegated to `next-themes`.

## Rule of thumb

Before adding new global state, ask: did this come from a fetch? Put it in a
React Query hook. Otherwise, is it read in more than a couple of unrelated
components? Add it to (or extend) a Context here. Component-local state
(form inputs, toggles, a single page's UI state) should stay as `useState`
in that component — it doesn't belong in either layer.

Do not duplicate the same value across a Context and a hook, or across two
Contexts (e.g. don't re-store `WalletContext.address` inside `AuthContext`) —
one of them should derive from the other instead.
