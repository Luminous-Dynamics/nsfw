# NSFW Video Tutorial Script

**Title**: "NSFW - Nix Package Management Made Easy on Windows"
**Duration**: 5-7 minutes
**Target Audience**: Windows developers new to Nix

---

## 🎬 Scene 1: Introduction (30 seconds)

### Visual
- Screen shows Windows desktop with PowerShell open
- NSFW logo/title appears

### Script
> "Hi! Today I'll show you NSFW - Nix Subsystem for Windows - a tool that makes it incredibly easy to manage Nix packages on Windows.

> If you've heard of Nix but found it intimidating on Windows, this is for you. NSFW bridges Windows and WSL2 to give you a simple, friendly interface to the powerful Nix package manager.

> Let's get started!"

---

## 🎬 Scene 2: Prerequisites (45 seconds)

### Visual
- Split screen showing:
  - Left: PowerShell running `wsl --version`
  - Right: Checklist appearing

### Script
> "Before we begin, you'll need three things already installed:

> First, Windows 10 or 11 with WSL2 enabled. Check by running `wsl --version`.

> Second, a WSL2 Linux distribution - Ubuntu is perfect.

> Third, Nix installed in your WSL2 environment.

> If you need help with these, check the links in the description. It takes about 10 minutes total."

---

## 🎬 Scene 3: Installation (30 seconds)

### Visual
- Terminal showing download and setup
- Binary appearing in directory

### Script
> "Installing NSFW is simple. Download the latest release from our GitHub page - it's a single executable, no installation needed.

> Just put `nsfw.exe` anywhere in your PATH, or in a convenient folder.

> That's it! You're ready to use it."

---

## 🎬 Scene 4: First Command - Search (60 seconds)

### Visual
- Terminal showing:
  ```
  nsfw search firefox
  ```
- Highlight the spinner appearing
- Highlight colored output
- Highlight results with version info

### Script
> "Let's search for Firefox. Type `nsfw search firefox`.

> Notice the spinner? It shows you what's happening - no more wondering if it's frozen.

> And look at this output! Package names in green, versions in yellow, clear formatting.

> NSFW found multiple Firefox variants - the regular browser, ESR edition, and developer edition. Each with its version and description.

> Now watch this - search again..."

### Visual
- Re-run the same search
- Highlight "Using cached results"
- Show instant response

### Script
> "See that? Instant results! NSFW caches searches for 5 minutes, so repeated searches are lightning fast."

---

## 🎬 Scene 5: Installing a Package (75 seconds)

### Visual
- Terminal showing:
  ```
  nsfw install vim
  ```
- Highlight interactive prompt
- Show progress spinner
- Show success message

### Script
> "Installing is just as easy. Type `nsfw install vim`.

> NSFW asks for confirmation - much nicer than before! Just press Y and Enter.

> The spinner shows installation progress. Nix is downloading and installing Vim in your WSL2 environment.

> And done! Green checkmark - Vim is installed.

> What if you try to install again?"

### Visual
- Run install command again
- Show "already installed" message

### Script
> "NSFW is smart - it tells you it's already there. No errors, just helpful info."

---

## 🎬 Scene 6: Listing Packages (30 seconds)

### Visual
- Terminal showing:
  ```
  nsfw list
  nsfw list --detailed
  ```

### Script
> "Want to see what's installed? Type `nsfw list`.

> Clean, numbered list. Add `--detailed` for version and store path information.

> Everything color-coded and easy to read."

---

## 🎬 Scene 7: Advanced Features (45 seconds)

### Visual
- Split screen showing:
  - JSON output: `nsfw search vim --format json`
  - Limit results: `nsfw search python --limit 5`
  - Skip prompts: `nsfw install neovim --yes`

### Script
> "NSFW has power features too.

> Need JSON for scripts? Add `--format json`.

> Too many results? Use `--limit` to show just what you need.

> Automating? Skip confirmations with `--yes`.

> Perfect for scripts and automation."

---

## 🎬 Scene 8: Removing Packages (30 seconds)

### Visual
- Terminal showing:
  ```
  nsfw remove vim
  ```
- Show confirmation and removal

### Script
> "Removing is just as easy. `nsfw remove vim`.

> Confirm, and it's gone. Clean and simple."

---

## 🎬 Scene 9: Error Handling (30 seconds)

### Visual
- Terminal showing a typo:
  ```
  nsfw install firefoxxx
  ```
- Highlight the error message with suggestion

### Script
> "Made a typo? NSFW gives you helpful errors.

> Not just 'package not found' - it suggests checking the name or updating channels.

> Errors that actually help you fix the problem."

---

## 🎬 Scene 10: Wrap-up (30 seconds)

### Visual
- Screen showing quick command reference
- Links appearing

### Script
> "And that's NSFW! Let's recap:

> Simple commands: search, install, list, remove.
> Beautiful colored output.
> Fast caching.
> Helpful errors.
> Windows-native but Nix-powered.

> Check the description for:
> - Download link
> - Full documentation
> - GitHub repository
> - Getting help

> Thanks for watching - happy package managing!"

---

## 📋 B-Roll Suggestions

Throughout the video, show:
- Code snippets appearing smoothly
- Spinners animating
- Colors highlighting important info
- Terminal output sliding in
- Checkmarks appearing for success
- Smooth transitions between commands

---

## 🎨 Visual Style Guidelines

### Colors
- Background: Dark theme (matches terminal)
- Accent: Cyan/blue (matches NSFW branding)
- Success: Green
- Warnings: Yellow
- Errors: Red

### Typography
- Monospace font for terminal
- Clean sans-serif for overlays
- Keep text large and readable

### Pace
- Let commands breathe (2-3 seconds per result)
- Don't rush through output
- Pause on important points

---

## 📱 Social Media Clips (30-second versions)

### Clip 1: "The Search"
- Show search with cache demo
- Hook: "This tool makes Nix searches INSTANT"
- CTA: "Full tutorial in bio"

### Clip 2: "Beautiful Output"
- Show before/after comparison
- Hook: "Look at this beautiful package management"
- CTA: "Download link in bio"

### Clip 3: "Smart Errors"
- Show helpful error message
- Hook: "Errors that actually HELP you"
- CTA: "Try NSFW today"

---

## 🎤 Voiceover Notes

### Tone
- Friendly and enthusiastic
- Professional but approachable
- Clear enunciation
- Moderate pace (not too fast)

### Emphasis Points
- "Lightning fast" (cache demo)
- "Just works" (installation)
- "Color-coded" (output)
- "Helpful errors" (error handling)

---

## 📊 Metrics to Track

After publishing:
- Watch time
- Drop-off points
- Repeat views (cache section usually replayed)
- Click-through to docs
- GitHub stars increase

---

## 🔄 Updates for Future Versions

When Phase 3 ships:
- Add section on configuration
- Show shell completions
- Demonstrate auto-update
- Profile management

Keep this script updated as a living document!

---

**Produced by Luminous Dynamics**
*Making NixOS accessible through consciousness-first technology*
