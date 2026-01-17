# Publishing mac-stats to GitHub

## Step 1: Create GitHub Repository

1. Go to https://github.com/new
2. Fill in the repository details:
   - **Repository name:** `mac-stats` (or your preferred name)
   - **Description:** macOS system stats monitoring app built with Tauri
   - **Visibility:** Choose Public or Private
   - **IMPORTANT:** Do NOT check:
     - ❌ Add a README file
     - ❌ Add .gitignore
     - ❌ Choose a license
   
   (We already have these files in the project)

3. Click **"Create repository"**

## Step 2: Add Remote and Push

After creating the repo, GitHub will show you commands. Use these (replace `YOUR_USERNAME` with your GitHub username):

```bash
# Add the remote repository
git remote add origin https://github.com/YOUR_USERNAME/mac-stats.git

# If you want to push the current branch (feat/theming)
git push -u origin feat/theming

# OR if you want to push to main instead:
git checkout main
git merge feat/theming  # Merge your changes into main
git push -u origin main
```

## Step 3: Verify

After pushing, verify it worked:

```bash
git remote -v  # Should show your GitHub repo
git branch -a  # Should show remote branches
```

## Optional: Create a README

If you don't have a README.md, you might want to create one:

```markdown
# mac-stats

macOS system stats monitoring app built with Tauri.

## Features

- Real-time CPU, RAM, Disk, and GPU monitoring
- Temperature readings (SMC)
- CPU frequency monitoring (IOReport)
- Process list with top CPU consumers
- Menu bar integration
- Modern, customizable UI

## Building

```bash
cd src-tauri
cargo build --release
```

## Requirements

- macOS (uses macOS-specific APIs)
- Rust
- Tauri CLI
```

## Current Status

- **Current branch:** `feat/theming`
- **Commits ready:** Multiple commits including recent optimizations
- **Remote:** Not configured yet (will be added in Step 2)
