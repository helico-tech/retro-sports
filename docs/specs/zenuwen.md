# STALEN ZENUWEN! (bike lifting in Amsterdam)

*"Amsterdam's fastest hands. Mind the BOA's."* Id `zenuwen`, module
`src/games/zenuwen/`. "Stalen zenuwen" is Dutch for nerves of steel, and
"stalen" is also "stole". Amsterdam has more bikes than people, and every
one of them has been stolen at least once. Your bike was stolen. Again.
Time to take one back.

The opponents are the BOA's (*buitengewoon opsporingsambtenaar*, the
city's uniformed enforcement officers, *handhavers*): they patrol the
canals on foot and by bike, and they write fines.

## The street
Side view along a canal (gracht), with a little depth like the dodgeball
court: gabled canal houses at the back, a brick street you walk on, the
quay edge with trees and parked bikes at the front, and the canal below
it. A stage is about four to five screens wide; the camera follows you.

## Stealing
- **Walk** with the arrows (up/down is depth). **Hold B** to run: faster,
  but running looks suspicious.
- Stand next to a parked bike and **press A** to work its lock. Each lock
  is a small game with the same buttons:
  - RINGSLOT (wheel lock, omafiets): tap A. Quiet.
  - KETTINGSLOT (chain, bakfiets): saw with LEFT, RIGHT, LEFT... Noisy.
  - BEUGELSLOT (U-lock, racefiets): a needle swings over a dial; press A
    in the green. A miss is a loud CLANK.
  - DUBBEL (the golden bike): a chain, then a U-lock.
  - Wrecks (ROESTBAK) are not locked: slow, squeaky getaway bikes.
  Moving away stops; the progress stays on that lock.
- **B at a lock**: the bolt cutter (KNIPTANG). Opens anything at once,
  with a KRAK every BOA within about 170 px hears (a ring shows how far).
  Two per stage.
- Opened: you are on the bike (GEJAT!). The bike is *hot* for a few
  seconds (a cut lock dangles from it).

## Riding and delivering
- Ride with the arrows. Each bike has its own speed (racefiets fast,
  bakfiets slow). Pedestrians in the way: you both fall over (noise).
- The quay has no railing. Hold DOWN at the edge and you ride into the
  canal (PLONS!): the bike is gone, you swim out.
- **SJAAK**, the fence, putters along the canal in a small boat piled
  with bikes. Ride to the quay next to his boat and **press A** to throw
  the bike on the pile. Cash!
- **C on a bike**: get off and leave it (innocent again).
- Every stage has an order (BESTELLING) of bike kinds. Deliver the whole
  order before the time runs out. Bikes not on the order still pay a
  little.

## The BOA's
- They walk (or ride) patrol routes and sometimes turn around to look.
  Their view is a cone on the street (shown as a faint pattern), blocked
  by trees, the krul (green iron urinal) and the glass bin.
- What they see fills their suspicion: picking a lock (fast), riding a
  hot bike, running. Walking is fine. A "?" over their head grows; at "!"
  they chase (and blow the whistle: colleagues nearby join).
- Noise (locks, the cutter, a crash, a startled heron) makes them come
  and look.
- **C on foot**: act casual (whistle, take a selfie). Nobody gets more
  suspicious, but you stand still. Next to a tree, the krul or the glass
  bin, C hides you, unless a BOA saw you go in.
- Lose them: break the line of sight for a few seconds, or jump into the
  canal (BOA's do not swim). On a bike you outrun BOA's on foot, not the
  one on a bike.
- Caught: BEKEURING! A fine (EUR 95), the bike goes back, and a short
  break. Three fines in a stage and you are OPGEPAKT (out).

## Thieves (select: SNEAK, HANDS, LEGS, WHEELS)
- JOOST, student: fast legs, clumsy hands. His bike was stolen seven
  times.
- OPA HARRIE, 78: nobody suspects Opa. Slow.
- SANNE, bike courier: rides anything, cannot open a jar.
- DE PROF, retired locksmith: opens anything, rides like a wardrobe.

## Stages (1P ladder)
1. PRINSENGRACHT, a quiet morning: HANDHAVER HENK (loves his coffee
   breaks) and a colleague. Ring locks and chains. Order of 4, 120 s.
2. CENTRAAL STATION: the huge bike parking by the water, tourists with
   suitcases, BOA FATIMA on a bike. More U-locks. Order of 5, 150 s.
3. KONINGSDAG (King's Day): an orange crowd (cover!), party boats, six
   BOA's including HOOFD-BOA DE BOER with his dog BRUNO, who sniffs out
   hiders. The mayor's golden bike is on the order. Order of 5, 150 s.
Clear the order to go on; out of time or three fines: continue screen.
Score: bike values, order bonus, time bonus, minus fines.

2P: thief versus BOA in split screen, two rounds on the same street with
the roles swapped. The thief earns money; the BOA earns EUR 95 per fine,
but fining someone who did nothing costs EUR 50 (KLACHT!). Most money
after both rounds wins.

Attract mode: a CPU thief on a random stage.

## Whimsy
Herons on the quay (they squawk and fly off when you ride past: noise),
a cat on a houseboat, coots nesting on a sunken bike, the tour boat, the
crane boat that fishes bikes out of the canal (and sometimes drops one
on the quay), a bike in a tree, party boats, Amsterdammertjes (the brown
bollards), BOA heckles (HE! JIJ DAAR!, DAT IS NIET JOUW FIETS!), banners
(GEJAT!, PLONS!, BEKEURING!, WEG WAS IE!).

## CPU and balance
`sim`: the CPU thief at three skill levels on every stage (clear rate,
time, fines, deliveries). `probe`: human-like bots (decent, reckless with
the cutter, sneaky, lazy) through the ladder. Targets:
- A decent player clears stage 1 almost always (90%+) and stage 3 about
  a third of the time.
- Sneaking pays: the reckless bot does clearly worse than the decent one.
- Ignoring the BOA's fails: the lazy bot does not clear stage 3.
- Waiting is not free: an over-careful bot runs out of time on stage 3.
- Stage times for a decent clear: about 30-60 s, 45-75 s, 90-130 s.

## As built (decisions)
- Suspicion per BOA fills only while they see you doing something: picking
  (fast), throwing a bike on the boat, riding a hot bike, running. "?"
  at 0.35 (they come and look), "!" at 1.0: a 30-frame whistle, then the
  run. After a chase starts you are GEZOCHT (wanted) for 7 s: any BOA who
  sees you then gets suspicious fast. The HUD shows GEZOCHT! and HEET!.
- A CPU BOA only fines a rider whose bike is hot (cut lock still dangling,
  5 s after the steal) or known stolen (seen in a chase). A fast rider
  squeezes past a surprised BOA, who then chases. Only a chasing BOA can
  grab a rider. 2P's BOA player may fine any rider (all your bikes are
  stolen), but an innocent costs a KLACHT.
- Losing them: out of sight for 2.5 s and the chaser searches, then gives
  up. Foot BOA's give up after 15 s (PFFF...). Found hiding while
  innocent: "KOM DAAR MAAR UIT", no fine.
- A BOA about to glance back shows a little arrow for 24 frames.
- The throw onto the boat cannot be interrupted. Sjaak moors away from
  BOA's, parked bikes and props, and casts off when a BOA walks up
  (POLITIE! WEG HIER!), so a BOA cannot camp his boat.
- A bike ridden into the canal is fished back (and parked) while the
  order still needs more of its kind than are left.
- Grace after a fine (4 s) ends when you start on a lock.
- Rare: a bike stuck in a tree (35% of streets), the crane boat that
  fishes a free wreck out of the canal (half of the streets).

## Balance results
Run with `cargo run --release -- --probe zenuwen 100` and
`--sim zenuwen 60` (2026-10-16).

| Target | Result |
| --- | --- |
| Decent clears stage 1 90%+ | 98% (37 s) |
| Decent clears stage 3 about a third | 36% (100 s); per thief 36-47% |
| Reckless clearly worse | 90% / 45% / 5% (decent 98 / 93 / 36) |
| Ignoring the BOA's fails | lazy 95% / 40% / 4% |
| Waiting is not free | sneaky 99% / 89% / 5% (runs out of time) |
| Stage 2 between | decent 93% (55 s) |

The CPU thief's skill (0.3-0.9) barely changes its results (it plays the
same plan with sharper timing); it only drives the attract demo.

