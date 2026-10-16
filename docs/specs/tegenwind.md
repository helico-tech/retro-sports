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
