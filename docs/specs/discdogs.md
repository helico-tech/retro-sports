# DISC DOGS! (frisbee)

*"The world's goodest frisbee championship."* Id `discdogs`, module
`src/games/discdogs/`. California Games' flying disc, except your dog does
the catching.

## The toss (one turn)
Side view, the camera scrolls right with the disc. A turn has three parts:
1. **Aim**: the owner stands at the throw line, the dog sits beside them
   wagging. Up/down sets the angle (0-45 degrees, a short dotted arc
   shows it). A windsock and an arrow show the wind for this toss.
2. **Power**: hold A; a power meter swings up and down. Release to throw.
   The top band is PERFECT (a small bonus and a sparkle). Holding C at
   release throws a FLOATER: higher, slower, much more wind drift.
3. **Fetch**: control switches to the dog. Left/right runs (the dog
   speeds up; double-tap for ZOOMIES, which drains the zoomies meter),
   B leaps. The dog catches when its mouth meets the disc. In the air,
   A does a flip; if the flip is not finished before landing the dog
   tumbles (OOF!) and drops it.

Disc physics: gravity, lift that grows with speed, drag, wind push. The
disc must feel floaty and readable: a shadow on the ground shows where it
is, and it wobbles as it slows.

## Scoring
Distance from the throw line where it is caught (in yards) times the
catch: ground or rolling catch x1, jump catch x2, flip catch x3. Bonus
for an over-the-shoulder catch (dog running away from the thrower) and a
PERFECT throw. A miss scores 0: the dog brings the disc back looking
guilty, or lies down in protest.

## Whimsy
- **Squirrel**: sometimes runs across the field. A dog close to it stops
  and stares unless you tap C (GOOD BOY!). Low-focus dogs stare longer.
- **Seagull** (pier stage): swoops at a slow disc. Catch it first or
  SEAGULL'D! (0 points, the gull flies off with it).
- **Picnic blankets**: running through one grabs a sausage (+5) but slows
  the dog. A man asleep under a newspaper; a kid with a kite whose string
  can block a high disc.
- **Pond**: a disc in the pond means a swimming dog and a shake-off.
- Title ticker jokes; owner and dog bios; the dog sulks when you miss
  three in a row.

## Dogs (select screen: speed, jump, focus, zoomies)
- BISCUIT, corgi: fast, low jump. "BUILT LOW TO THE GROUND. UNSTOPPABLE."
- TURBO, border collie: all-round. "HAS A PHD IN FRISBEE."
- SIR WOOFINGTON, afghan hound: huge jump, slow, low focus.
  "HAIR BY VIDAL BASSOON."
- KEES, keeshond: top focus, stocky. "LOVES CHEESE. FEARS NOTHING."
Each owner is a `person()` with its own hair style and colors.

## Stages and modes
1P ladder, three stages, each against one CPU owner and dog. Both sides
throw 5 times, alternating; the higher total wins; continue on a loss.
1. SUNNY HILL PARK (afternoon, trees, picnics, squirrel).
2. PIER 92 (sunset boardwalk, sea, seagulls).
3. THE GRAND BOWL (night stadium, lights, crowd, gusty wind).

2P: take turns on the same stage, each with their own dog.
Attract mode: CPU vs CPU tosses.

## CPU and balance
CPU thrower picks angle and power with noise; CPU dog runs to the
predicted landing spot and leaps with timing noise; flips depend on
skill. Headless `sim`: CPU vs CPU by skill (catch rate, flip success,
average toss). `probe`: scripted human bots (safe: never flips; greedy:
always flips; decent). Targets:
- CPU catch rate 60-85% from easy to hard; a typical toss 20-60 yards.
- Decent probe wins stage 1 about 90%, stage 3 about 35%.
- Always flipping is not better than playing safe on average.

## Sound
Songs: a bouncy title song, a happy play song, a win jingle. Effects:
throw whoosh, bark, catch chomp, flip whoosh, oof, splash, seagull
squawk, squirrel chitter, wind gust, meter tick, perfect ding, whistle,
cheer, menu blip and confirm.
