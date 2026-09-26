# Keeping this fork current

`main` is the single working branch for this fork. It contains the NFSMW patches and the other development branch histories. Historical branches remain available as references; new work belongs on `main`.

The `upstream` remote tracks athei/mtld3d independently of the fork's working branch.

## Clone once

```sh
git clone git@github.com:g-cqd/mtld3d.git
cd mtld3d
git remote add upstream https://github.com/athei/mtld3d.git
git switch main
```

## Integrate an upstream update

Start with a clean working tree, then merge without rewriting the fork's history:

```sh
git fetch upstream --tags
git switch main
git merge --no-commit --no-ff upstream/main
```

Resolve conflicts and run the repository's build and test gates, including the NFSMW regressions, before committing the merge. `git merge --abort` returns an interrupted merge to its starting point. Keep installed game runtimes separate from validation builds.

Push verified changes to `origin/main` with a normal push. Do not force-push or delete the historical branches as part of an update.
