# Sovereign Tech Fund Application — `faction`
### Draft matching the official form (apply.sovereigntechfund.de), field by field

*Word counts noted are the form's stated limits. Draft text below is written to fit within them — trim further if the live portal enforces strictly. Replace bracketed notes with final numbers/links before submission.*

---

### Application name
`faction` — protocol-agnostic distributed systems bootstrapping and membership

### Acknowledgement checkboxes
- Subscribe to broadcast + notification emails: **check this**, or you won't hear back.
- Legal capacity to sign: confirm — you're applying as an individual.
- FOSS licensing: `faction` is MIT-licensed — satisfies this.
- No duplicate public/private funding for the same activities: confirm honestly. If any Ethereum-ecosystem grant (Rocket Pool, LEGO, Obol) comes through for overlapping Phase 1–4 work before this is reviewed, that changes what you can truthfully check here — revisit at submission time.

---

### Project title
`faction`: a formally-tested, `no_std` Mealy state machine for distributed cluster membership

---

### Describe your project in a sentence. (100 words)

`faction` is a protocol-agnostic, `no_std + alloc`, zero-unsafe Rust crate implementing cluster bootstrapping, peer discovery, and dynamic membership as a fully static Mealy state machine, independently publishable and reusable across any distributed system — from blockchain validator sets to embedded industrial control clusters — rather than reimplemented per-project. Phase 0 is complete: 275+ tests, 100% state-input coverage, a CRAP score of 0, zero unsafe code, and verified `no_std` compatibility across 10+ spawn/transport combinations, all under an adversarial-testing methodology. This application requests support for Phases 1–4: dynamic joining, failure detection, and safe single-node addition/removal.

*(~95 words)*

---

### Describe your project more in-depth. Why is it critical? (300 words)

Every distributed system — blockchain validator networks, industrial control clusters, IoT fleets, cloud orchestration platforms — depends on the same unglamorous foundation: nodes must discover each other, join a running cluster without corrupting its shared state, detect failures, and leave cleanly. In practice, nearly every project reimplements this logic independently, at varying levels of rigor, coupled tightly to its own protocol. Examples in wide production use — etcd's Raft-based membership changes, Consul's Serf gossip layer, libp2p's discovery mechanisms — each solve a version of this problem, but none are `no_std`-compatible or usable as a reusable, protocol-agnostic component outside their own ecosystem.

`faction` (originally `cabal`, renamed for a crates.io conflict) addresses this by implementing the membership/bootstrapping problem exactly once, with a level of rigor most infrastructure code never receives: a fully static Mealy state machine with 100% state-input coverage, zero unsafe code, and full `no_std` compatibility so it runs identically on embedded microcontrollers and cloud servers alike.

This is critical because membership logic sits below the application layer, where correctness failures are silent until a cluster partitions or a node join corrupts shared state — exactly the class of bug that's expensive to discover in production and nearly impossible to retrofit test coverage onto after the fact. `faction`'s adversarial testing methodology (100% state × input coverage, CRAP score of 0) is designed to catch these failure modes before they reach any consumer, rather than relying on each downstream project to independently achieve the same rigor.

Phase 0, the protocol-agnostic core, is complete and published. This application covers Phases 1–4, extending the same rigor to dynamic joining, failure detection, and node addition/removal under real churn and partition conditions.

*(~260 words)*

---

### Link to project repository
[crates.io/crates/faction — insert exact URL]

### Link to project website
[if none exists, either leave blank or point to the GitHub repo README]

---

### Provide a brief overview of your project's own, most important, dependencies. (300 words)

`faction` is deliberately minimal in its dependency surface, consistent with its `no_std + alloc` design goal and zero-unsafe requirement.

[Umberto — fill in concretely: does `faction` depend on anything beyond core/alloc? e.g. any `heapless`, any allocator abstraction, any specific async runtime coupling (should be none, given protocol-agnostic design), any macro crates for the state machine derive, etc. If dependency count is genuinely near-zero, say so explicitly — STF's evaluators read a minimal, deliberate dependency footprint as a maturity signal, especially for a `no_std` embedded-target crate.]

*(fill to ~150–250 words with actual crate list and rationale for each)*

---

### Provide a brief overview of projects that depend on your technology. (300 words)

`faction`'s primary current consumer is EtheRAM, the author's formally-specified (TLA+), embedded Ethereum-like blockchain implementing IBFT and Raft consensus, validated on real NUCLEO-F767ZI/H743ZI hardware clusters — not simulation. EtheRAM uses `faction` for its cluster bootstrapping and membership layer across both its IBFT and Raft consensus implementations.

Beyond this, `faction` is newly published and does not yet have external adopters. [Be honest here — STF weighs prevalence heavily, but a young, rigorously-built component with one serious internal consumer and a clear path to reuse is a legitimate answer; overstating adoption is easy to check and costs credibility. If there's any early interest — GitHub stars, issues, discussions, anyone who's said they're evaluating it — include that concretely.]

*(~100 words as drafted — expand with any real adoption signal you have)*

---

### Which target groups does your project address (who are its users?) and how would they benefit? (300 words)

Primary target groups:

- **Embedded and distributed-systems engineers** building clusters on resource-constrained hardware (industrial control, energy monitoring, IoT fleets), who currently have no `no_std`-compatible, independently-tested membership primitive and must build one from scratch or forgo formal rigor.
- **Blockchain and consensus-protocol implementers** who need a bootstrapping/membership layer decoupled from their specific consensus logic, avoiding the common anti-pattern of tightly coupling network membership to protocol semantics.
- **Safety-critical and formally-verified systems teams**, who benefit directly from a component with 100% state-input coverage and zero unsafe code as a component they can integrate without re-deriving its correctness guarantees themselves.

Direct benefit: adopting `faction` means inheriting its adversarial test coverage and `no_std`/zero-unsafe guarantees rather than re-implementing and re-testing membership logic per project. Indirect benefit: as more systems adopt a shared, rigorously tested membership layer instead of ad hoc reimplementations, the overall reliability baseline of distributed infrastructure — particularly in embedded/industrial contexts where failures have physical consequences — improves.

*(~170 words)*

---

### Describe a specific scenario for the use of your technology and how this meets the needs of your target groups. (300 words)

A concrete scenario, drawn from the author's own EtheRAM project: a 5-to-7-node cluster of NUCLEO-F767ZI/H743ZI embedded boards, connected via a managed Ethernet switch, running IBFT consensus. When a new board is added to the physical network (Phase 1/3 scope) or an existing board crashes and needs to be detected and excluded (Phase 2/4 scope), the cluster's shared view of membership must update correctly under real network conditions — packet loss, ARP stalls, timing jitter — without corrupting in-flight consensus state.

This is precisely the scenario `faction`'s Phase 1–4 work targets: dynamic joining and failure detection validated not just in simulation, but against the same class of real embedded hardware constraints (limited memory, `no_std` environments, real network non-determinism) that make this problem hard in practice. An embedded-systems team building an industrial sensor cluster faces an identical structural problem — nodes joining and leaving a physical network — even though their consensus/application layer is entirely different from IBFT. `faction`'s protocol-agnostic design means the same tested membership logic applies to both.

*(~175 words)*

---

### How was work on the project made possible so far? Other funding sources? (300 words)

`faction`/Phase 0 was developed independently, self-funded, alongside the author's paid contract work (currently an embedded C# role at Hexagon/Leica Geosystems, held primarily for financial stability rather than as the author's primary technical focus). No external grant or investment has been received for `faction` or EtheRAM to date.

The author has applied for Ethereum-ecosystem grant funding (Rocket Pool GMC Round 38, LEGO/Lido Ecosystem Grants, Obol Labs, and initially Ethereum Foundation's RFP process) covering Phases 1–4 of this same roadmap, framed for those programs' blockchain-specific mandates. As of this application, [state current status honestly: e.g. "Rocket Pool has responded and a formal application is under their review; other programs were found closed, paused, or did not proceed" — update precisely at submission time]. No funding commitment has been finalized from any of these sources.

*(~140 words — tighten/update with exact current status before submission)*

---

### What are the challenges you currently face in the maintenance of the technology? (300 words)

As a solo-maintained project, the primary challenges are capacity and sustainability rather than technical debt: all development, adversarial test-suite expansion, and quality tooling (`crap4rust`, `test-gap-gate`, `embedded-gate`) has been built and maintained by a single person alongside full-time contract work. This creates structural risk — bus-factor risk on a project whose entire value proposition rests on sustained rigor, and a hard ceiling on how much of the Phase 1–4 roadmap (dynamic joining, failure detection, node add/remove, each requiring the same adversarial-testing discipline as Phase 0) can be delivered without dedicated funded time.

Other maintenance challenges typical of an early-stage but rigorously-built project: no external contributor base yet to share review or triage load; no formal governance structure needed yet given single-maintainer status, but this will need to be addressed if/when external contributions begin; and the ongoing need to keep `no_std` compatibility verified across an expanding spawn/transport test matrix as new transports or targets are added in later phases.

*(~155 words)*

---

### What are possible alternatives to your project and how does it compare? (300 words)

Existing alternatives solve adjacent but not identical problems:

- **etcd / Consul (Serf)** — mature, widely deployed membership/discovery solutions, but neither is `no_std`-compatible or usable on embedded/resource-constrained targets; both assume a full OS environment.
- **libp2p's discovery/peer-routing stack** — flexible and widely used in the P2P/blockchain space, but general-purpose and heavyweight rather than formally minimal; not designed around 100% state-input coverage or zero-unsafe guarantees as first-class goals.
- **Protocol-specific membership logic** (e.g., custom membership code embedded directly in a given consensus implementation) — the status quo for most blockchain and distributed systems projects; tightly coupled to the specific protocol, not reusable, and rarely subjected to the level of adversarial testing `faction` applies.

`faction`'s differentiation is the combination of three properties rarely found together: `no_std` embedded compatibility, protocol-agnostic reusability, and formally adversarial test rigor (100% state-input coverage, zero unsafe code, CRAP score 0). Existing alternatives typically offer at most one or two of these three simultaneously.

*(~155 words)*

---

### What do you plan to implement with STF support? (900 words)

Phase 0 recap. faction's completed core answers a narrower question than "membership management" broadly: is a statically-known group of nodes ready to proceed? It is a two-phase startup barrier coordinating participation and readiness quorum, fully deterministic, observable, and queryable — validated across 10 system-test combinations spanning 3 spawn models (Task, Thread, Process) and 4 transport protocols (Memory, Channels, TCP, gRPC), with 275 tests, 100% (state, command) coverage, a CRAP score of 0, and zero unsafe code. This application requests support to extend that same rigor from static readiness-tracking into full dynamic membership lifecycle management: Phases 1 through 4.

Every phase is bound by constraints that never relax as the machine grows: the crate stays no_std + alloc so the same binary runs on bare metal, WASM, embedded RTOS, and cloud; zero unsafe code throughout; the core Mealy invariant output = F(state, input) holds as a pure function with no side effects; every (state, input) pair is explicitly tested, not sampled; the machine remains protocol-agnostic with no Raft- or IBFT-specific knowledge leaking in; every transition, query, and rejection is observable; and each phase's full test suite must still pass unchanged six phases later — a structural regression guarantee, not a policy.

Phase 1 — Dynamic joining (4–6 weeks). Removes the static-peer-list limitation. A peer can send a JoinRequested signal at runtime and be admitted via JoinApproved/JoinRejected outcomes, after which its inputs are treated as valid member signals. Critically, faction itself never decides admission policy — it surfaces the request via EmitJoinRequest and acts strictly on the caller's decision, keeping the crate protocol-agnostic. This is the first attack surface this application specifically targets: a join mechanism is inherently exposed to malformed or adversarial join attempts, and the phase does not begin implementation until its full (state, input) test matrix is defined; it is not complete until that matrix passes at 100%.

Phase 2 — Failure detection (4–6 weeks, depends on Phase 1). Introduces SWIM-style liveness probing (suspect → indirect probe → confirm or revive) and three new states — Stable, Degraded(SuspectSet), QuorumLost — with quorum re-evaluated continuously as liveness changes. As with joining, faction never performs probing itself; it emits SendProbe/SendIndirectProbe commands and consumes the results (ProbeAck, ProbeMiss, indirect variants) as inputs, preserving the Mealy purity invariant. This phase converts the crate from tracking a fixed membership set to actively monitoring a live one — the necessary foundation for the safe addition/removal protocols in Phases 3–4.

Phase 3 — Single-node addition (4–6 weeks, depends on Phase 2). This is deliberately distinct from Phase 1's join-gatekeeping: Phase 1 answers "is this peer allowed to participate," permissively and without coordination guarantees; Phase 3 answers "does changing the member set preserve quorum safety," via a commit/abort reconfiguration protocol with a structural single-change-at-a-time guard — it becomes impossible, not merely disallowed, to have two additions in flight simultaneously. The core safety invariant: at no point may two disjoint subsets of nodes each independently believe they hold a valid quorum.

Phase 4 — Single-node removal (4–6 weeks, depends on Phase 3). Symmetric to Phase 3 but with a harder invariant: removals that would drop the live set below quorum threshold are structurally unavailable as a state transition, not merely rejected by a runtime check. This phase also explicitly treats degenerate cases as first-class, tested transitions rather than error paths — removing an already-suspected-dead node, attempting to remove the last node in a minority partition (correctly rejected as WouldBreakQuorum), and probes arriving after a node has already been removed (silently and correctly discarded via a defined transition).

Cross-cutting deliverables, all four phases. Each phase is gated: 100% (state, input) coverage is mandatory before the next phase begins, not an end-of-project aspiration. Concretely, this application commits to: (1) maintaining a CRAP score of 0 as the input/state space grows through all four phases; (2) preserving zero unsafe code throughout; (3) re-validating full no_std compatibility across the existing spawn/transport matrix as each phase lands; and (4) enforcing the strict-superset property — Phase 0's 275 tests, and every subsequent phase's tests, continue passing unchanged as later phases are added, giving a mechanically checkable regression guarantee rather than a documentation promise.

Beyond Phase 4, the existing roadmap defines two further phases — membership epochs/rejoin handling (Phase 5) and concurrent membership changes (Phase 6) — which are not part of this specific funding request but represent the project's committed direction, underscoring that Phases 1–4 are a coherent, already-planned unit of work rather than a scope invented for this application.

*(Pull directly from your existing internal roadmap docs — this is the one section where your prior grant-writing work transfers almost word for word.)*

---

### How many hours do you estimate for these activities?
[Number only — estimate from actual EtheRAM/faction development velocity so far. If Phase 0 took N hours for a comparable scope, use that as your basis rather than guessing cold.]

### Estimate the cost of the work in EUR (numbers only)
[Must exceed €50,000. Convert your existing $100,000 ask at a current rate, or better, cost it bottom-up from the hours estimate above at a stated hourly rate — STF explicitly evaluates budget feasibility against the activities, so a rate-derived number is more defensible than a round figure carried over from a different application.]

### In how many months will you perform the activities?
[Number only — your own realistic estimate given this is likely part-time alongside the Hexagon contract, unless STF funding would let you go full-time on it.]

---

### Who would be most qualified to implement this work, and why? (300 words)

The author is the sole author and maintainer of both `faction` and its primary consumer, EtheRAM, and is the only person with complete context on the design decisions behind Phase 0's state machine and testing methodology. Relevant qualifications: 15+ years of software engineering experience across medical technology, finance, defense, and embedded systems; an MSc in Computer Engineering (La Sapienza, Rome, 110/110); iSAQB and ICP-ACC certifications. Direct, demonstrated delivery on this exact codebase — Phase 0 is complete, published, and independently verifiable on crates.io, not a proposal without prior output. Formal-methods background evidenced by EtheRAM's TLA+ specification and its IBFT/Raft consensus validated on real embedded hardware clusters, not simulation alone. The custom quality tooling built specifically to enforce this project's standards (`crap4rust`, `test-gap-gate`, `embedded-gate`) was authored in-house rather than adopted off the shelf, reflecting sustained, specialized investment in exactly the kind of rigor this proposal commits to maintaining through Phases 1–4.

*(~140 words)*

---

### Your name/handle
Umberto Gotti

### Link to your profile (optional)
[GitHub profile URL]

### What is your role in this project?
**Maintainer**

### If not maintainer, are you in contact with the maintainer/community?
N/A — you are the maintainer.

### Country of residence of the person who will sign the contract
Switzerland

### How did you hear about the Sovereign Tech Agency? (optional)
[Answer honestly — doesn't affect selection]

---

## Pre-submission checklist

1. Fill every `[bracketed]` placeholder above with real numbers/links before pasting into the portal — several fields (dependencies, hours, cost, months) can't be responsibly estimated without your input.
2. Recompute the EUR figure at submission time, not now — exchange rates drift and STF wants a EUR-native number, not a converted USD one.
3. Be exact and current on the "other funding sources" and "duplicate funding" questions — check the live status of Rocket Pool/LEGO/Obol outreach right before submitting, since STF explicitly disqualifies projects receiving parallel public/private funding for the *same* activities.
4. The "projects that depend on your technology" field is the weakest as drafted, since `faction` is young — resist the temptation to inflate it; an honest "one serious internal consumer, early-stage otherwise" answer is more credible than vague claims of broader adoption.
5. The 900-word implementation section is where your existing EF/Rocket Pool grant-writing content transfers most directly — start there to save time, then strip Ethereum-specific integration language.
