# Upstream Synchronization Guide

Sharkord Plus is designed to maintain 100% backward compatibility with upstream [Sharkord](https://github.com/Sharkord/sharkord). When new features, security updates, or bug fixes are merged into the upstream repository, they can be selectively incorporated into Sharkord Plus.

The repository's git remote structure is configured as follows:

- **origin** -> `https://github.com/HomelessAvatar/sharkord_plus.git` (Sharkord Plus repository)
- **upstream** -> `https://github.com/Sharkord/sharkord.git` (Official Sharkord repository)

---

## Synchronization Workflow

To fetch and selectively integrate upstream changes:

### 1. Fetch Upstream Branches
```bash
git fetch upstream
```

### 2. Inspect Recent Upstream Changes
```bash
# View the latest commits on upstream's development branch
git log upstream/development --oneline -n 20

# View details of a specific commit
git show <commit-hash>
```

### 3. Cherry-Pick Specific Fixes or Features
To adopt a single bug fix or enhancement without pulling unintended upstream changes:
```bash
git cherry-pick <commit-hash>
```
*If merge conflicts arise, resolve them, verify with `bun run magic` / `cargo test --all`, and run `git cherry-pick --continue`.*

### 4. Merge an Upstream Branch (Full Synchronization)
To perform a complete branch merge:
```bash
git merge upstream/development
```

This workflow ensures Sharkord Plus stays independent, modular, and performant while effortlessly benefiting from upstream innovations.
