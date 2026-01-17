# How to Change Default Branch on GitHub

## Step-by-Step Instructions

### 1. Navigate to Repository Settings

Go to your repository's branch settings:
**https://github.com/raro42/mac-stats/settings/branches**

Or manually:
1. Go to: https://github.com/raro42/mac-stats
2. Click on **Settings** tab (top right)
3. Click on **Branches** in the left sidebar

### 2. Change Default Branch

1. In the **"Default branch"** section, you'll see the current default branch (`feat/theming`)
2. Click the **switch/edit icon** (pencil icon) next to the default branch name
3. A dropdown will appear with available branches
4. Select **`main`** from the dropdown
5. Click **"Update"** button
6. Confirm the change if prompted

### 3. Delete Old Branch (Optional)

After changing the default branch, you can delete `feat/theming`:

1. Go to: https://github.com/raro42/mac-stats/branches
2. Find `feat/theming` in the list
3. Click the **trash icon** next to it
4. Confirm deletion

## Alternative: Using GitHub CLI

If you have GitHub CLI installed:

```bash
gh api repos/raro42/mac-stats -X PATCH -f default_branch=main
```

## Verification

After changing the default branch:
- The repository homepage will show `main` as the default
- New clones will checkout `main` by default
- Pull requests will target `main` by default

## Current Status

- ✅ `main` branch exists and is up-to-date
- ✅ `feat/theme` branch exists (same as main)
- ⚠️ `feat/theming` is still the default (needs to be changed)
