# ProofCat vs. the incumbents

> **Every claim about another vendor links to that vendor's own public documentation.**
> If a cell says "not documented", it means exactly that — we could not find it stated
> publicly, not that the feature is absent. If we are wrong, open an issue with a
> source and we will correct this page.
>
> **Sources checked: 13 July 2026.** Vendors ship updates; this page can go stale.

## Who's who

| Tool | Vendor | What it is |
|---|---|---|
| **ProofCat** | this repo | Verified card offload + metadata inspection. Open source (MIT), free, offline. |
| **OffShoot** | Hedge | The market-standard verified offload tool. macOS + Windows + iOS. |
| **FoolCat** | Hedge | Camera *reports* (thumbnails + metadata). Not an offload tool. Pairs with OffShoot. |
| **Silverstack XT / Offload Manager** | Pomfort | On-set data management; offload is one part of a larger suite. |
| **ShotPut Pro** | Imagine Products | Long-standing offload tool with PDF reports. |

## Scope of the replacement claim

ProofCat is a feature-level replacement for the **core verified-offload workflow**:
copying a card to multiple destinations, independent read-back, fail-closed
`SAFE_TO_FORMAT` gating, resume/repair and machine-readable evidence. It is not a
claim to replace the whole Hedge product family or its ecosystem: FoolCat reports,
EditReady, cloud/integration services, commercial support, years of field history
and vendor RAW colour pipelines remain outside this scope.

## The offload core

| | ProofCat | OffShoot (Hedge) | Silverstack / Offload Manager | ShotPut Pro |
|---|---|---|---|---|
| **Refuses to bless a card when both destinations sit on the same physical disk** | ✅ core rule | not documented | not documented | not documented |
| **Fail-closed on destination mismatch** | ✅ withholds `SAFE_TO_FORMAT` on *any* warning | ❌ documented: "the transfer will continue and be completed with warnings" [1] | not documented | not documented |
| Independent destination read-back | ✅ always, in ArchiveMax profile | ⚙️ optional — only in "Source & Destination" mode, "takes twice as long" [1] | ⚙️ verification is a setting [4] | ⚙️ selectable verification types [5] |
| Default mode verifies… | full hash of source **and** destination | **file sizes only** ("Transfer" mode) [1] | checksum during copy; verification toggleable [4] | selectable [5] |
| **BLAKE3 (cryptographic hash)** | ✅ | not documented [1] | not documented [3] | not documented [5] |
| Two hashes computed in one pass | ✅ XXH64 + BLAKE3 | not documented | not documented | not documented |
| xxHash | ✅ XXH64 | ✅ XXH64BE, always [1] | ✅ [3][4] | ✅ [5] |
| MD5 / SHA-1 / C4 | — (not planned) | ✅ legacy, optional [1] | ✅ MD5, SHA1 [3] | ✅ MD5, SHA [5] |
| ASC MHL manifest per destination | ✅ | ✅ (MHL *verification* is a Pro-tier feature) [2] | ✅ [3] | not confirmed |
| Multiple destinations at once | ✅ | ✅ | ✅ [3] | ✅ |
| Resume after crash / disconnect | ✅ | ✅ stop/resume [2] | not confirmed | not confirmed |
| Automatic re-copy after a later MHL verification issue | ✅ targeted repair from a verified source/replica | ✅ OffShoot documents automatic MHL re-verification and a retry from the source or another destination [1] | not documented | not documented |
| Machine-readable verdict (`SAFE_TO_FORMAT`) | ✅ | ❌ no equivalent documented | ❌ no equivalent documented | ❌ no equivalent documented |

### The one row that matters

Every tool here copies and checksums. **The difference is what happens when something
is off, and what the tool is willing to *promise*.**

OffShoot's own documentation states that when a destination verification issue is
found during the selected verification mode, "the transfer will continue and be
completed with warnings" [1]. OffShoot also documents a separate automatic MHL
re-verification step that can retry a file from the source or another destination
[1]. These are useful safeguards, but they are not the same as a fail-closed,
machine-readable erasure gate: the operator still has to interpret the warning
state and decide whether the card is safe to wipe.

ProofCat makes that decision explicit and refuses to hedge: `SAFE_TO_FORMAT` is
granted **only** when every one of these holds —

- the copy landed on **two distinct physical devices** (not two folders on one disk);
- every destination was **independently re-read from disk** and hashed;
- an **ASC MHL** manifest was written to each destination;
- **zero warnings** of any kind.

Any doubt at all → the verdict is withheld. The card stays.

## Price & licensing

| | ProofCat | OffShoot | FoolCat | Silverstack / OM | ShotPut Pro |
|---|---|---|---|---|---|
| Price | **free** | $169 · Pro $249 · $49/30 days [2] | $89 · Pro $129 · $29/mo [6] | subscription — see vendor store [3] | see vendor store |
| Licence | **MIT, open source** | perpetual, closed, 1 yr updates [2] | perpetual, closed, 1 yr updates [6] | closed | closed |
| Account / cloud required | **no — fully offline** | no (cloud features optional) | no | no (ShotHub optional) | no |
| Source code auditable | **✅ yes** | ❌ | ❌ | ❌ | ❌ |

For a tool that tells you it is safe to erase the only copy of a shoot day,
"you can read the code that made that decision" is not a small thing.

## Speed

Measured **on our own test rig only** — a MacBook (Apple Silicon) writing to a USB SSD.
We do **not** publish head-to-head speed claims against other tools: throughput depends
on the card, the reader, the cable and the destination, so a cross-tool number measured
on different hardware would be meaningless.

| ProofCat profile | What it does | Throughput |
|---|---|---|
| **Fast** | durable copy, no read-back | ≈ **386 MB/s** |
| **ArchiveMax** | copy + independent destination read-back + XXH64 + BLAKE3 | ≈ **310 MB/s** |

**Full verification costs about 20%.** Reproduce it yourself: `docs/benchmark-protocol.md`.

## Where the incumbents are still ahead — honestly

We are not going to pretend otherwise.

| | Who wins | Why |
|---|---|---|
| **Years in production** | Hedge, Pomfort, Imagine | Thousands of DITs, years of odd hardware, edge cases we have not met yet. ProofCat is new. This is earned with time, not code. |
| **RAW colour science** | FoolCat Pro, Silverstack | RAW → Rec.709 conversion and 3D LUTs via camera-vendor SDKs (ARRIRAW, R3D, X-OCN) [6]. We are **not** going to chase this. |
| **Camera reports with thumbnails** | FoolCat | Rich HTML/PDF reports across ARRIRAW, R3D, BRAW, CinemaDNG, ProRes RAW, X-OCN [6]. ProofCat: planned, not shipped. |
| **Ecosystem** | Hedge, Pomfort | iconik, S3, Codex engines, ShotHub, bundles, integrations [2]. We have none. |
| **Commercial support** | all of them | You get a support contract. With ProofCat you get GitHub issues and me. |

If you need RAW colour pipelines or a support SLA, buy their software. It is good software.
ProofCat exists for one job: **prove the copy is real before the card gets wiped.**

## Sources

1. Hedge — OffShoot verification modes, checksum algorithms, mismatch behaviour: <https://docs.hedge.video/offshoot/features/verification>
2. Hedge — OffShoot product page: features, tiers, pricing: <https://hedge.co/products/offshoot>
3. Pomfort — Offload Manager: checksum methods, ASC MHL: <https://pomfort.com/offloadmanager/>
4. Pomfort — Silverstack copy & verification behaviour: <https://kb.pomfort.com/silverstack/silverstack-hands-on/offloadandbackup/copy-verification-process-silverstack-verification-behavior/>
5. Imagine Products — ShotPut Pro: <https://www.imagineproducts.com/product/shotput-pro> · spec sheet: <https://www.imagineproducts.com/storage/544/ShotPut-Pro-Spec-Sheet-2025.pdf>
6. Hedge — FoolCat product page: features, formats, pricing: <https://hedge.co/products/foolcat>

*Found an error? Open an issue with a source link. This page gets corrected, not defended.*
