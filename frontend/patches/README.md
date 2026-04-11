# Frontend Dependency Patches

This directory stores emergency patches for frontend dependencies when `package.json` `overrides` are not enough.

## Rules

1. prefer `overrides` first
2. add a patch only when version forcing cannot fix the issue
3. register every patch in `frontend/dependency-patches.json`
4. include a removal condition for every patch
5. retire the patch as soon as upstream publishes a safe fix

## Patch File Format

Patch files must be standard `git diff` style patches whose paths are relative to the patched package root.

They are applied by:

```bash
cd frontend
node scripts/apply-dependency-patches.mjs
```

## Typical Workflow

1. install dependencies with the normal project workflow
2. modify the affected files under `frontend/node_modules/<package>`
3. create a patch file under `frontend/patches/`
4. add an entry to `frontend/dependency-patches.json`
5. verify the patch applies cleanly in a fresh install
