# Demo Evidence Report — S-3.7.02

**Story:** Claroty fixture generator — all 8 archetypes (`prism-dtu-claroty`, D-060)
**Impl SHA:** `32a71acd`
**Recorded:** 2026-04-28
**Tests:** 24/24 GREEN (workspace unchanged)

---

## Coverage Map

| BC | VP | AC | Recording |
|----|----|----|-----------|
| BC-3.4.001 | VP-108 | Seeded RNG primitive idempotent | [BC-3.4.001-004-claroty-generator-24-green.gif](BC-3.4.001-004-claroty-generator-24-green.gif) |
| BC-3.4.001 | — | Distinct seeds produce distinct records | same |
| BC-3.4.001 | — | Distinct org_ids produce distinct records | same |
| BC-3.4.001 | — | Seed MAX u64 does not panic | same |
| BC-3.4.001 | — | Sequential determinism calls identical | same |
| BC-3.4.002 | VP-112 | schema_valid non-drift archetypes | same |
| BC-3.4.002 | VP-113 | Schema drift flag + first record invalid | same |
| BC-3.4.002 | VP-114 | Schema validation gated behind cfg(test) | same |
| BC-3.4.002 | — | Dormant tenant empty records trivially valid | same |
| BC-3.4.003 | — | Healthy OT environment baseline counts | same |
| BC-3.4.003 | — | Auth outage baseline and first call is 401 | same |
| BC-3.4.003 | — | Compromised endpoint baseline + severity | same |
| BC-3.4.003 | — | Schema drift baseline count | same |
| BC-3.4.003 | — | High churn baseline + tombstones | same |
| BC-3.4.003 | — | Pagination edge cases counts + cursors | same |
| BC-3.4.003 | — | Dormant tenant zero records at any scale | same |
| BC-3.4.003 | — | Scale 0.1 healthy OT environment counts | same |
| BC-3.4.003 | — | Large scale baseline counts + subnets | same |
| BC-3.4.003 | — | Dormant tenant zero records + cursors | same |
| BC-3.4.004 | VP-119 | Disjoint ID sets for different orgs | same |
| BC-3.4.004 | VP-120 | Alert IDs carry org slug prefix | same |
| BC-3.4.004 | VP-120 | Device IDs carry org slug prefix | same |
| BC-3.4.004 | — | Unregistered org returns error | same |
| BC-3.4.004 | — | Tombstone IDs follow tomb format | same |

**Recording:** `BC-3.4.001-004-claroty-generator-24-green.gif` (432 KB)
**Tape:** `BC-3.4.001-004-claroty-generator-24-green.tape`
**Command:** `cargo test -p prism-dtu-claroty --features fixture-gen --test bc_3_4_claroty_generator`
**Result:** 24 tests, 0 failures

---

## Files

| File | Size |
|------|------|
| `BC-3.4.001-004-claroty-generator-24-green.gif` | 432 KB |
| `BC-3.4.001-004-claroty-generator-24-green.webm` | 816 KB |
| `BC-3.4.001-004-claroty-generator-24-green.tape` | 646 B |
