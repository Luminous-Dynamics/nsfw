# 🚀 Custom Package Index - Making Search Actually Fast

**Problem**: Nix search is fundamentally slow (2-10 minutes) because it evaluates entire nixpkgs tree.

**Solution**: Build our own pre-processed package index for instant search.

**Timeline**: 1-2 days of focused work

**Impact**: Transforms NSFW from "slow but powerful" to "fast AND powerful"

---

## The Problem in Detail

### Current Flow (Slow)
```
nsfw search firefox
  → wsl nix search nixpkgs firefox
    → Nix evaluates entire nixpkgs tree (80,000+ packages)
    → First time: 2-10 minutes
    → Repeat searches: Still slow for new terms
```

### Why Nix is Slow
1. **Lazy evaluation**: Nix doesn't have pre-built index
2. **Full tree traversal**: Every search evaluates all packages
3. **Flakes cache helps**: But only for exact repeat searches
4. **Fundamental design**: Nix optimizes for correctness, not search speed

### Why This Blocks "Best on Windows"
- Chocolatey search: <1 second
- winget search: <1 second
- NSFW search: 2-10 minutes ❌

**Users will not tolerate this.** They'll try it once, wait 5 minutes, and uninstall.

---

## The Solution: Custom Package Index

### Build Pre-Processed Index

**Concept**:
- Periodically scrape nixpkgs (daily/weekly)
- Extract: name, version, description, homepage, license
- Store in SQLite database (fast, portable)
- NSFW searches this index instead of calling Nix

**Result**: Search is instant (<100ms)

### Implementation Options

#### Option A: SQLite Index (Recommended)
**Pros**:
- Fast (indexed queries <100ms)
- Portable (single file)
- No dependencies (SQLite built into Rust)
- Can ship with NSFW binary

**Cons**:
- Need to update index periodically
- Slightly stale data (but refreshable)

**Storage**: ~10-20MB for full nixpkgs index

#### Option B: JSON Index
**Pros**:
- Simple to generate
- Easy to inspect
- No dependencies

**Cons**:
- Slower (linear search through 80k packages)
- Larger file size
- Need to load entire file into memory

#### Option C: Embedded Search Engine (Tantivy)
**Pros**:
- Professional full-text search
- Fuzzy matching
- Relevance ranking

**Cons**:
- More complex
- Larger binary size
- Overkill for our needs

**Recommendation**: Start with **Option A (SQLite)**

---

## Implementation Plan

### Phase 1: Index Generator (4-6 hours)

**Build tool that generates index**:

```bash
# Script: generate-index.sh
#!/bin/bash

# Query nixpkgs for all packages (run once, takes 10 minutes)
nix search nixpkgs --json > nixpkgs-all.json

# Parse and load into SQLite
cargo run --bin generate-index -- nixpkgs-all.json packages.db
```

**SQLite Schema**:
```sql
CREATE TABLE packages (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT,
    pname TEXT,
    description TEXT,
    homepage TEXT,
    license TEXT,
    UNIQUE(name)
);

CREATE INDEX idx_name ON packages(name);
CREATE INDEX idx_pname ON packages(pname);
CREATE INDEX idx_description ON packages(description);
```

**Rust Code**:
```rust
// src/bin/generate-index.rs
use serde_json::Value;
use rusqlite::{Connection, params};

fn main() -> Result<()> {
    let json = fs::read_to_string("nixpkgs-all.json")?;
    let data: Value = serde_json::from_str(&json)?;

    let conn = Connection::open("packages.db")?;
    conn.execute(CREATE_TABLE_SQL)?;

    for (name, pkg) in data.as_object().unwrap() {
        conn.execute(
            "INSERT INTO packages (name, version, pname, description) VALUES (?1, ?2, ?3, ?4)",
            params![
                name,
                pkg["version"].as_str(),
                pkg["pname"].as_str(),
                pkg["description"].as_str(),
            ],
        )?;
    }

    Ok(())
}
```

### Phase 2: Search Implementation (2-3 hours)

**Update NSFW to use index**:

```rust
// src/nix_ops/package_index.rs
use rusqlite::{Connection, params};

pub struct PackageIndex {
    conn: Connection,
}

impl PackageIndex {
    pub fn new() -> Result<Self> {
        // Embedded index shipped with binary
        let conn = Connection::open_in_memory()?;

        // Load from embedded data
        let index_data = include_bytes!("../packages.db");
        conn.restore("main", index_data)?;

        Ok(Self { conn })
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Package>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, version, description
             FROM packages
             WHERE name LIKE ?1 OR description LIKE ?1
             ORDER BY
                CASE
                    WHEN name LIKE ?2 THEN 0
                    WHEN name LIKE ?1 THEN 1
                    ELSE 2
                END
             LIMIT ?3"
        )?;

        let pattern = format!("%{}%", query);
        let exact_pattern = format!("{}%", query);

        let packages = stmt.query_map(
            params![pattern, exact_pattern, limit],
            |row| {
                Ok(Package {
                    name: row.get(0)?,
                    version: row.get(1)?,
                    description: row.get(2)?,
                })
            }
        )?;

        Ok(packages.collect()?)
    }
}
```

**Update search command**:
```rust
// src/cli/commands.rs (search function)

pub fn search(query: &str, limit: usize) -> Result<()> {
    let progress = ProgressIndicator::spinner("Searching 70,000+ packages...");

    // Search local index (instant!)
    let index = PackageIndex::new()?;
    let results = index.search(query, limit)?;

    progress.finish_and_clear();

    // Display results
    display_results(results);

    Ok(())
}
```

### Phase 3: Index Updates (2-3 hours)

**Allow users to update index**:

```bash
# Update index from latest nixpkgs
nsfw update-index

# Shows current index age
nsfw index-info
```

**Implementation**:
```rust
pub fn update_index() -> Result<()> {
    let progress = ProgressIndicator::spinner("Fetching latest nixpkgs (this may take 10 minutes)...");

    // Run nix search to get all packages
    let output = Command::new("wsl")
        .args(&["nix", "search", "nixpkgs", "--json"])
        .output()?;

    progress.set_message("Building search index...");

    // Parse and rebuild index
    let packages: Value = serde_json::from_slice(&output.stdout)?;
    rebuild_index(packages)?;

    progress.finish_and_clear();
    eprintln!("✅ Index updated! {} packages indexed", count);

    Ok(())
}
```

### Phase 4: Fallback to Nix (1 hour)

**If local index doesn't have package, fall back to live Nix search**:

```rust
pub fn search(query: &str, limit: usize) -> Result<()> {
    // Try local index first
    let index = PackageIndex::new()?;
    let results = index.search(query, limit)?;

    if results.is_empty() {
        // No results in index, try live Nix search
        eprintln!("⚠️  Not in local index, searching nixpkgs (this may take 2-10 minutes)...");
        return search_nix_live(query, limit);
    }

    display_results(results);
    Ok(())
}
```

---

## Timeline & Milestones

### Day 1 (6-8 hours)
- ✅ Morning: Design schema and test with small dataset
- ✅ Afternoon: Build index generator
- ✅ Evening: Generate full nixpkgs index, test search speed

### Day 2 (4-6 hours)
- ✅ Morning: Integrate into NSFW search command
- ✅ Afternoon: Add update-index command
- ✅ Evening: Test, document, polish

### Deliverable
- 🎯 Search time: 2-10 minutes → <100ms (100x-6000x faster!)
- 🎯 User experience: "Just works" instantly
- 🎯 Competitive: Matches Chocolatey/winget speed
- 🎯 Maintainable: Easy to update index

---

## Index Maintenance Strategy

### Frequency
**Ship with index**: Include pre-built index in release (weekly builds)
**User updates**: `nsfw update-index` (optional, for latest packages)
**Auto-update**: Check weekly, prompt user to update

### Index Age
- ✅ 1 week old: Fresh, no warning
- ⚠️ 1 month old: "Your index is 1 month old, run `nsfw update-index`"
- ❌ 3 months old: "Your index is outdated, please update"

### CI/CD Integration
```yaml
# .github/workflows/build-index.yml
name: Build Package Index
on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly
  workflow_dispatch:

jobs:
  build-index:
    runs-on: ubuntu-latest
    steps:
      - name: Generate index
        run: |
          nix search nixpkgs --json > nixpkgs-all.json
          cargo run --bin generate-index
      - name: Upload artifact
        uses: actions/upload-artifact@v2
        with:
          name: packages.db
          path: packages.db
```

---

## Expected Results

### Performance Comparison

| Operation | Before | After | Improvement |
|-----------|--------|-------|-------------|
| **First search** | 2-10 min | <100ms | 1000x-6000x faster |
| **Repeat search** | 2-10 min | <100ms | 1000x-6000x faster |
| **Different term** | 2-10 min | <100ms | 1000x-6000x faster |
| **User experience** | ❌ Broken | ✅ Instant | Night & day |

### Competitive Position

| Package Manager | Search Speed | Our Speed |
|-----------------|--------------|-----------|
| Chocolatey | <1 second | <100ms ✅ |
| winget | <1 second | <100ms ✅ |
| Nix direct | 2-10 minutes | <100ms ✅ |

**Result**: We're now competitive on speed while maintaining our 70k package advantage.

---

## Trade-offs & Considerations

### Pros ✅
- **100x-6000x faster** searches
- **Competitive** with Chocolatey/winget
- **Offline search** (works without internet)
- **Better UX** (feels professional)
- **Portable** (single SQLite file)

### Cons ⚠️
- **Slightly stale** (but updateable)
- **Extra complexity** (need to maintain index)
- **Larger binary** (extra ~10-20MB)
- **Build step** (generate index in CI/CD)

### Why Pros Outweigh Cons

**For "best on Windows" goal**: We MUST have instant search. No mainstream user will tolerate 2-10 minute searches. This is non-negotiable.

**Staleness is acceptable**:
- Nix packages don't change that often
- Users can update-index weekly
- Much better than unusable search

**Complexity is manageable**:
- SQLite is stable and well-understood
- Generate index once in CI/CD
- Ship pre-built index with releases

---

## Alternative: Hybrid Approach

**Could we do both?**

1. **Default**: Search local index (instant)
2. **If no results**: Offer to search live Nix
3. **If very new package**: User runs update-index

**Example**:
```bash
$ nsfw search brand-new-package
⚡ Searching local index... (0.05s)
ℹ️  No results in local index (last updated 3 days ago)

❓ Search live nixpkgs? (takes 2-10 minutes) [y/N]
```

This gives us:
- ✅ Speed for 99% of searches
- ✅ Completeness for edge cases
- ✅ User choice and control

---

## Recommendation for VM Claude

### Immediate Next Steps

1. **Prototype**: Build minimal index generator (2 hours)
   - Generate small test index (100 packages)
   - Test search speed
   - Validate approach

2. **Validate**: Compare to live Nix (1 hour)
   - Does it find same packages?
   - Is it actually faster?
   - Any missing data?

3. **Decide**: Ship it or refine? (30 minutes)
   - If prototype works: Build full version
   - If issues: Adjust approach
   - If fundamental problem: Try different solution

4. **Implement**: Full custom index (4-6 hours)
   - Generate full nixpkgs index
   - Integrate into NSFW
   - Add update-index command
   - Test thoroughly

5. **Document**: Update README (1 hour)
   - "Instant search of 70,000 packages"
   - Explain update-index
   - Competitive advantage highlighted

---

## Impact on "Best on Windows" Goal

**Before custom index**:
- ❌ 2-10 minute searches block mainstream adoption
- ❌ Can't compete with Chocolatey on speed
- ❌ Users think it's broken

**After custom index**:
- ✅ Instant search makes it feel professional
- ✅ Competitive with Chocolatey on speed
- ✅ Can claim "70,000 packages, instant search"
- ✅ No longer a blocker to mainstream adoption

**This is critical infrastructure for v0.3.0.**

---

## Conclusion

**VM Claude, you're absolutely right**: UX improvements don't fix the fundamental slowness. We need the custom index.

**Go for Option 3.** This is 1-2 days well spent.

**Priority**: This is now the #1 blocker for mainstream adoption. Let's solve it properly.

**Autonomy**: You have full authority to implement this however you think best. The approach above is just a suggestion.

**Support**: If you hit any blockers, document them and we'll solve together.

---

**Status**: Custom index approved, VM Claude has green light

**Timeline**: 1-2 days of focused work

**Impact**: Transforms NSFW from "slow but powerful" to "fast AND powerful"

🚀 **Let's make search actually fast!**
