Here’s a crisp, ship-ready implementation plan to bake the MVP weighting into Omnigraph. It’s scoped, ordered, and wired end-to-end (ingest → GDS → composites → HUD), with clear DoD per milestone.

# Omnigraph MVP Insight Weights — Implementation Plan

## 0) Goals (what “done” looks like)

* Compute fast, interpretable signals that answer: **what’s core**, **what’s risky**, **what’s a chokepoint**, and **what has best refactor payoff**.
* Run at 60 fps on the canvas; all heavy math lives in Neo4j GDS or in batched background steps.
* Surface results in the **HUD** (microfacts + peek) and in the **Properties** panel.
* Deterministic export/import of metrics for reproducibility.

---

## 1) Graph Model & Ingest

### 1.1 Entities & Relations

* **Nodes**

  * `Module {path, loc, complexity, churn30, lastChangeDays, owners, coverage}`
  * `Function {fqname, loc, complexity}` (optional for MVP; focus on Module first)
* **Relationships**

  * `(:Module)-[:IMPORTS {weight:int}]->(:Module)`
  * `(:Function)-[:CALLS {weight:int}]->(:Function)` (optional day-2)
* **Weights**

  * `IMPORTS.weight = numSymbolsImported` (fallback 1)
  * `CALLS.weight = callFrequency` (fallback 1)

### 1.2 Ingestion Pipeline (Rust → Neo4j)

* Extract repo features (git log) and complexity (parser/linter).
* Upsert with Cypher params; batch by 5–10k records.

```cypher
UNWIND $modules AS m
MERGE (n:Module {path:m.path})
SET n.loc=m.loc, n.complexity=m.cyclo, n.churn30=m.churn30,
    n.lastChangeDays=m.lastChangeDays, n.owners=m.owners,
    n.coverage=m.coverage;

UNWIND $imports AS r
MATCH (a:Module {path:r.src}), (b:Module {path:r.dst})
MERGE (a)-[e:IMPORTS]->(b)
SET e.weight = coalesce(r.weight, 1);
```

**DoD:** Nodes/edges present with repo features attached; idempotent re-ingest.

---

## 2) GDS Graph Projections

### 2.1 Projections

* **Imports graph** (primary):

```cypher
CALL gds.graph.project(
  'og_imports',
  {Module: {}},
  {IMPORTS: {orientation:'NATURAL', properties:'weight'}}
);
```

* **Calls graph** (optional day-2):

```cypher
CALL gds.graph.project(
  'og_calls',
  {Function: {}},
  {CALLS: {orientation:'NATURAL', properties:'weight'}}
);
```

**DoD:** Projections created/refreshed in <2s for 10k nodes (target).

---

## 3) Core Algorithms (MVP set)

Run on `og_imports` unless noted.

1. **PageRank** (imports & calls):

```cypher
CALL gds.pageRank.stream('og_imports', {relationshipWeightProperty:'weight'})
YIELD nodeId, score
WITH gds.util.asNode(nodeId) AS n, score
SET n.pr_imports = score;
```

(Repeat for `og_calls` if projected → `n.pr_calls`.)

2. **In-degree / degree**

```cypher
MATCH (n:Module)<-[e:IMPORTS]-()
WITH n, count(e) AS indeg
SET n.indeg_imports = indeg;
```

3. **Louvain** (communities)

```cypher
CALL gds.louvain.stream('og_imports')
YIELD nodeId, communityId
WITH gds.util.asNode(nodeId) AS n, communityId
SET n.community = communityId;
```

4. **Approx Betweenness** (chokepoints)

```cypher
CALL gds.betweenness.stream('og_imports', {sampleSize:4000})
YIELD nodeId, score
WITH gds.util.asNode(nodeId) AS n, score
SET n.btwn = score;
```

5. **k-Core** (backbone)

```cypher
CALL gds.kcore.stream('og_imports')
YIELD nodeId, coreValue
WITH gds.util.asNode(nodeId) AS n, coreValue
SET n.kcore = coreValue;
```

6. **Clustering** (tangledness)

```cypher
CALL gds.localClusteringCoefficient.stream('og_imports')
YIELD nodeId, localClusteringCoefficient
WITH gds.util.asNode(nodeId) AS n, localClusteringCoefficient
SET n.clustering = localClusteringCoefficient;
```

7. **WCC** (islands)

```cypher
CALL gds.wcc.stream('og_imports')
YIELD nodeId, componentId
WITH gds.util.asNode(nodeId) AS n, componentId
SET n.component = componentId;
```

**Optional day-4:** articulation points / bridges (or approximate via degree+betweenness).

**DoD:** Metrics persisted on nodes; runtime profiles captured.

---

## 4) Normalization & Composite Scores

### 4.1 Normalization strategy (transparent, bounded)

* Per metric, compute min/max over `:Module`; store normalized fields `*_norm ∈ [0,1]`.
* Use **robust min/max** (trim 1% tails) to reduce outlier skew.

```cypher
// Example: normalize pr_imports
MATCH (n:Module) WITH percentileCont(n.pr_imports,0.01) AS lo,
                        percentileCont(n.pr_imports,0.99) AS hi
MATCH (n:Module)
WITH n, lo, hi, case when hi=lo then 1.0 else (n.pr_imports-lo)/(hi-lo) end AS z
SET n.pri_norm = gds.util.clamp(z, 0.0, 1.0);
```

Repeat for: `pr_calls`, `kcore`, `indeg_imports`, `clustering`, `btwn`,
and repo features: `churn30`, `complexity`, `owners`, `coverage`.

### 4.2 Composite formulas (stored on node)

**importance**

```
importance = 0.35*pri_norm + 0.25*prc_norm + 0.15*kcore_norm
           + 0.10*indeg_norm + 0.15*clustering_norm
```

**chokepoint**

```
chokepoint = 0.6*btwn_norm + 0.2*articulation_flag + 0.2*bridge_flag
```

**risk**

```
risk = 0.30*churn_norm + 0.25*complexity_norm + 0.20*owners_norm
     + 0.15*chokepoint + 0.10*(1 - coverage_norm)
```

**payoff**

```
payoff = risk * importance
```

Implement as one Cypher pass or in Rust post-read; persist results to `n.importance`, `n.chokepoint`, `n.risk`, `n.payoff`.

**DoD:** All nodes have normalized metrics + four composites; top-K queries return stable rankings.

---

## 5) Queries & APIs (Rust ≤→ TS)

### 5.1 Cypher Queries

* **Top important modules**

```cypher
MATCH (n:Module) RETURN n.path AS path, n.importance AS s
ORDER BY s DESC LIMIT $k;
```

* **High payoff targets**

```cypher
MATCH (n:Module) WHERE n.payoff > $threshold
RETURN n.path, n.payoff ORDER BY n.payoff DESC LIMIT $k;
```

* **By selection (for HUD)**

```cypher
MATCH (n:Module {path:$path})
RETURN n {
  .path, .community, .importance, .risk, .chokepoint, .payoff,
  .pri_norm, .kcore, .clustering, .indeg_imports,
  .churn30, .complexity, .owners, .coverage
} AS m;
```

### 5.2 Rust service

* `get_node_metrics(path: &str) -> NodeMetrics`
* `top_by(metric: Metric, k: u32) -> Vec<NodeMetrics>`
* `layout_summary() -> LayoutStats` (counts, communities, etc.)

### 5.3 TS client

* `metricsStore.set(await api.getNodeMetrics(path))`
* HUD pulls from store; Properties panel shows full detail; color by `community`.

**DoD:** Endpoints return in <50 ms for single node; <300 ms for top-K.

---

## 6) HUD & Panel Integration

### 6.1 Micro-HUD (selection)

* Show: **name**, **community color swatch**, **importance**, **risk** (two tiny bars), actions: **Inspect**, **Jump to dependents**.
* Peak info under **Alt-hold**: add **chokepoint** + **payoff** bars, top 3 dependents/dependees list.

### 6.2 Properties Panel

* Full table of raw + normalized metrics, composite breakdown, sparkline of churn30 (if available).

### 6.3 Visual encodings

* **Node color** = `community`
* **Node size** = `importance` (clamped range)
* **Node halo** (thin glow) if `chokepoint > 0.7`
* **Risk icon** badge if `risk > 0.7`
* **“Payoff” tag** in list views (top 5%)

**DoD:** Selection updates HUD in <1 frame; encodings reflect recomputed values.

---

## 7) Performance & Scheduling

* Heavy steps (betweenness, louvain) **batch at load** or on **“Recompute Metrics”** command.
* Use **approx betweenness** with `sampleSize` tuned to dataset (start 4000).
* Cache normalized ranges to avoid repeated percentile scans.
* Throttle recomputes; compute composites in Rust if that’s cheaper for you.

**DoD:** Full compute (imports-only) finishes in seconds on 10k nodes; canvas render time unaffected.

---

## 8) Testing & Validation

* **Unit**: normalization clamps; composite math matches spec.
* **Property tests**: monotonicity (increasing `risk` inputs always increases `risk`).
* **Golden samples**: small repo fixture with known rankings.
* **Profiling**: GDS run times; endpoint latencies.

**DoD:** CI passes; golden rankings stable; p95 latency budgets met.

---

## 9) Milestones (1-week MVP cadence)

**Day 1–2:**

* Ingest + imports projection
* PageRank, degree, Louvain
* Normalize + `importance`
* HUD: importance + community color

**Day 3–4:**

* k-Core, clustering
* Approx betweenness → `chokepoint`
* Compose `risk` with churn/complexity/owners/coverage
* Properties panel wired

**Day 5:**

* `payoff` + top-K queries
* Visual encodings (size/halo/badges)
* Export metrics snapshot JSON

**Stretch (Weekend):**

* Calls graph + `pr_calls`
* Bridges/articulation flags
* Layout report (communities summary)

---

## 10) Acceptance Criteria (MVP)

* Canvas stays 60 fps while selecting and opening HUD/panels.
* For a 10k-module repo:

  * Imports projection < 2 s
  * PageRank < 1 s
  * Louvain < 2 s
  * Approx betweenness < 5 s (tunable)
* HUD shows **importance** and **risk** within 100 ms of selection.
* Top-K “payoff targets” renders a stable list; export/import round-trips.

---

## 11) File Map (new/updated)

```
src/
├─ services/
│  ├─ neo4j_client.rs            # pooled driver, query helpers
│  ├─ metrics_compute.rs         # composites, normalization (alt: Cypher)
│  └─ api.rs                     # get_node_metrics, top_by
├─ ingestion/
│  ├─ git_signals.rs             # churn, recency, owners
│  └─ complexity.rs              # cyclomatic, LOC
├─ cypher/
│  ├─ project_imports.cypher
│  ├─ alg_pagerank.cypher
│  ├─ alg_louvain.cypher
│  ├─ alg_betweenness_approx.cypher
│  ├─ alg_kcore.cypher
│  ├─ alg_clustering.cypher
│  ├─ normalize.cypher
│  └─ composites.cypher
├─ ui/
│  ├─ hud/micro-hud.ts           # bars + actions
│  ├─ panels/properties.ts       # metric tables
│  └─ encodings.ts               # color/size/badges
└─ utils/
   └─ stats.ts                   # min-max (robust), clamp, percentiles
```

---

## 12) Future-proofing (not in MVP, easy add-on later)

* Time-sliced metrics (trend lines of risk/importance).
* “What-if” explorer (remove a node; recompute local impact).
* Ownership bus factor (Herfindahl index over authors).
* Coverage-weighted PageRank (down-rank untested dependencies).

—

This gives you a lean, legible analytics core that tells the truth without black boxes—and plugs straight into your HUD aesthetic. Next natural step after MVP: record a “metrics snapshot” per commit so you can animate architectural drift over time like a time-lapse of a codebase growing moss.
