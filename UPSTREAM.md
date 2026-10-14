# Keeping this fork current

This fork uses a single `main` branch. It contains the merged NFSMW patches and maintains history from the previous development lineage. The transient branches have been removed; their commits remain reachable from `main`.

The `upstream` remote tracks athei/mtld3d independently of the fork's working branch.

## Clone once

```sh
git clone git@github.com:g-cqd/mtld3d.git
cd mtld3d
git remote add upstream https://github.com/athei/mtld3d.git
```

## Integrate an upstream update

Start with a clean working tree on `main`, then merge without rewriting the fork's history:

```sh
git fetch origin && git rebase origin/main
git fetch upstream --tags
git merge --no-commit --no-ff upstream/main
```

Resolve conflicts and run the repository's build and test gates, including the NFSMW regressions, before committing the merge. `git merge --abort` returns an interrupted merge to its starting point. Keep installed game runtimes separate from validation builds.

Push verified changes to `origin/main` with a normal push. Never force-push. The fork uses only the `main` branch; integrate upstream updates here and commit directly to main.
