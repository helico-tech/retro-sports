# TEGENWIND! (Dutch cycling)

*"Holland's toughest bike race. Against the wind."* Id `tegenwind`,
module `src/games/tegenwind/`. "Tegenwind" is Dutch for headwind; every
Dutch child has cycled to school against it, both ways.

## Riding
Side-scrolling race to the right on a bike path with two lanes (near and
far, with a little depth like the dodgeball court).
- **Pedal**: alternate LEFT and RIGHT. Every correct alternation is a
  stroke. Strokes faster than the cadence limit for your speed are wasted
  (the legs spin, TRAPPEN!), so rhythm beats mashing.
- **Up/down**: change lane.
- **B**: hop over potholes, tram rails, a herring on the road, a sleeping
  cat.
- **A**: ring the bell (TRING TRING!). Tourists in your lane step aside.
  Geese do not; they hiss and chase you.
- **Hold C**: tuck low. Much less drag (big in a headwind), but no bell,
  no hop, slower lane changes.
- **Drafting**: right behind another rider in the same lane the drag
  drops a lot; wind streaks show it. CPU riders draft too.
- **Wind**: the route turns, so the wind changes per section: a sign
  shows TEGENWIND (head), WIND MEE (tail) or ZIJWIND (side, pushes you
  across lanes). Gusts show as streaks; windmills spin faster.
- **Energy**: pedalling uses energy; empty means THE MAN WITH THE HAMMER
  (Dutch for bonking) and a much lower cadence limit. Ride through
  pickups to eat: stroopwafel, kroket from a snack bar, a herring.
- **Cargo**: every stage you carry something absurd on the rack: a crate
  of herring, a tulip pot, a tuba, a cheese wheel, a grandfather clock.
  Late hops, potholes and crashes knock pieces off. Delivered cargo
  scores a bonus.
- **Crash**: hit something fast and you fly over the handlebars into the
  canal (PLONS!), climb out dripping, 2 s lost. Banners: DOE NORMAAL!

## Bikes (select: speed, wind, cargo, hop)
- OMAFIETS (granny bike): upright, lots of drag, great cargo.
- RACEFIETS (road bike): light and fast, cargo falls off easily.
- BAKFIETS (cargo bike): slow, stable, huge cargo bonus.
- LIGFIETS (recumbent): lowest drag, terrible hop, can't see over hedges.
The rider is a `person()` in a new cycling pose; legs animate with the
pedals.

## Rivals (CPU)
- OMA RIEK on her omafiets with groceries: never stops pedalling,
  unbeatable in a headwind, says nothing.
- DENNIS in full lycra: fast with tailwind, hopeless in headwind.
- FAMILIE DE VRIES: a bakfiets with two kids and a dog in front; slow but
  drafts like a pro.

## Stages and modes
1P ladder; finish in the top 2 to go on, win the last one. Each stage
about 90-150 s.
1. DE POLDER: windmills, cows, sheep crossing, light headwind.
2. AMSTERDAM: canals, cobbles (screen shake), tram rails to hop,
   tourists to ring at, a drawbridge that opens (wait, or make it).
3. AFSLUITDIJK: the 32 km dike; storm headwind, sea spray, sheep, one
   lonely snack bar halfway.
2P: split screen (P1 top, P2 bottom, `Frame::copy_band`) with two CPU
riders. Attract mode: CPU riders.

## Whimsy
HUP HOLLAND HUP banners, cheese market, herring cart, a sudden rain
shower (an umbrella pickup: more drag, but dry), cows that moo as you
pass, geese, a cyclist with a sofa on her bike, Dutch exclamations
(LEKKER BEZIG!, GEZELLIG!, DOE NORMAAL!).

## CPU and balance
`sim`: CPU races (finish times, gaps, how often Oma Riek wins). `probe`:
human bots (max mash, steady cadence, drafter). Targets:
- Stage times 90-150 s; the field finishes within about 15 s.
- Steady cadence beats mashing; drafting is worth it; tucking matters
  in headwind.
- Decent probe gets top 2 on stage 1 about 90%, wins stage 3 about 30%.

## Sound
Songs: a street-organ style title song, an upbeat polka play song, a
fanfare. Effects: bell, pedal tick, wind gust, splash, moo, goose honk,
crash, hop, cobble rattle, chomp, start whistle, tram bell, bridge alarm,
cheer, menu blip and confirm.

## As built (decisions)
- Cadence: the limit is per bike (10 frames between strokes), lower while
  getting going and +7 when bonked. A press before the limit is wasted
  (no push, -0.7% speed, energy, and the next push can't be PERFECT); a
  clean push within 3 frames after the limit is PERFECT (+20%, costs more
  energy). A speed-dependent beat (the spec's "for your speed") made the
  rhythm unreadable, so it became a per-bike limit. The HUD meter (TRAP)
  and a gold glow on the crank show the PERFECT window.
- Both pedal keys held for 10+ frames = brake (for bollards, the bridge).
- Bikes: OMAFIETS all-rounder (good hop, 4 cargo), RACEFIETS lowest
  rolling drag but suffers on cobbles, fragile cargo, RACEFIETS and
  LIGFIETS lowest air drag tucked; LIGFIETS lowest drag upright, poor hop,
  and no look-ahead warning signs (it can't see over hedges); BAKFIETS
  slowest but bump- and crash-proof, steady in side wind, 6 cargo at 250.
- Look-ahead: yellow warning diamonds at the right screen edge for
  trouble up to 150 px off screen, a red light for the bridge alarm.
- Drawbridge (Amsterdam): opens when the leading human (or the leader in
  a CPU race) comes within 200 px of the barrier; a sprint can still make
  it before the boom drops; then about 5 s of waiting.
- Rain shower (polder or Amsterdam, 60% of races): grab an umbrella
  before it or get wet (energy drains). An open umbrella is a sail: more
  drag into the wind, a push with it.
- Cargo: cheese (polder), tulips (Amsterdam), herring (dike); after a
  continue you carry tubas, then grandfather clocks.
- Extras: cheese stand and herring cart with snacks, the lady with the
  sofa and the tractor (slow riders you can draft), geese that chase you
  when you ring at them, a rare stork / party boat / kite surfer, a
  photo finish banner, a sweeper van (BEZEMWAGEN) for whoever is still out
  45 s after the last CPU.
- 2P: a three-stage tour against two CPU rivals; most points wins.
  Points: place 1000/600/300/100, cargo, 2 per PERFECT, 25 per snack,
  50 for a swimming diploma (any trip into the canal).

## Balance results
Run with `cargo run --release -- --sim tegenwind 200` and
`--probe tegenwind 20` (2026-10-16).

| Target | Result |
| --- | --- |
| Stage times 90-150 s | decent human 117 / 116 / 150 s; CPU winner 131 / 117 / 147 s |
| Field finishes within about 15 s | CPU races: average spread 7 / 16 / 13 s (max 25-40 s) |
| How often Oma Riek wins | 12% polder, 12% Amsterdam, 40% on the stormy dike (Dennis 0% there) |
| Steady cadence beats mashing | masher: 68% presses wasted, 4.5-6.8 bonks, polder 214 s vs 117 s, swept up on stages 2-3 |
| Drafting is worth it | no-draft bot: polder 79% wins vs 86%, dike 18% vs 26% |
| Tucking matters in a headwind | no-tuck bot on the dike: 155 s, 10% wins vs 150 s, 26% |
| Decent probe: top 2 on stage 1 about 90% | 96% (86% wins) |
| Decent probe: wins stage 3 about 30% | 26% (54% top 2) |
| Every bike can win | dike wins: omafiets 30%, racefiets 25%, bakfiets 15%, ligfiets 35% |

Rival personalities in CPU-only races: De Vries (drafting) wins the
polder 38%, Dennis (tailwind) wins Amsterdam 36%, Oma Riek (headwind)
wins the dike 40%.
