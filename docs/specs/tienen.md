# TIENEN! (street football)

*"Ten points. One goal. The loser gets the billenbrand."* Id `tienen`,
module `src/games/tienen/`. Tienen ("tenning") is the Dutch street
football game every kid played behind the flats: one goal, one keeper,
everybody else tries to score on him, and every kid starts with ten
points. Rules from
[nl.wikipedia.org/wiki/Tienen_(voetbalspel)](https://nl.wikipedia.org/wiki/Tienen_(voetbalspel)).

## The pitch
Side view with depth, like the dodgeball court: one small goal on the
left (posts at the goal line, a net behind it, a wall behind that), the
pitch to the right, one screen wide, chalk lines on the stones. In front
of the goal a chalk box: the keeper's area (KEEPERSVAK). The background
is the venue (flats, the school, the cage), the front edge a kerb with
props. The ball has a shadow, so you can always see where it will land.

## The rules (as played)
- Four kids. One is the keeper, the other three play together against
  him. Every kid has their own points: they start at 10, the first keeper
  at 11. Points only go down while you are the keeper.
- A goal only counts **from the air**: the ball must come through the
  air from someone else (a cross, a keeper's parry) and be shot before
  it touches the ground. A goal takes points off the keeper:
  - VOLLEY (TRAP) 1, CHEST (BORST) 1
  - HEADER (KOPBAL) 2
  - HEEL (HAKJE) 2
  - BICYCLE KICK (OMHAAL) 5
- **Keeper change (WISSEL!)**: the field player who touched the ball last
  goes in goal when
  - the ball bounces before it goes in (GESTUITERD!),
  - the ball touches the ground outside the pitch (UIT!; still in the air
    outside the lines is fine),
  - the keeper catches it inside his area (GEVANGEN!).
  The old keeper becomes a field player and keeps his points.
- Exactly zero and you lose. Overshoot (a 5 on a keeper with 2) and the
  keeper goes back to ten minus what was left over (DOORGESCHOTEN! 2 - 5
  = -3, back to 7). So near the end you pick your shot: a keeper on 1
  can only be beaten with a volley.
- A keeper on 1 point may catch everywhere, not only in his area.
- Who keeps first: LATJETRAP. Every kid gets a shot at the crossbar;
  hit it and you are safe. Up to three rounds among the misses; still
  more than one left: AFTELLEN (a counting-out rhyme picks one).
- The loser gets the BILLENBRAND: he stands bent over in front of the
  goal and the others get a shot at his bottom.

## Controls
Field player:
- Arrows: run. Facing follows the arrows: face the goal (left) for a
  volley or header, turn your back (right) for a heel or bicycle kick.
- A: SHOOT at the goal. What comes out depends on the ball's height
  when you touch it: feet = volley (or heel with your back to the goal),
  chest = chest shot, head = header, in a jump with your back to the goal
  and the ball above your waist = bicycle kick. Up/down aim at the far
  or near post. On a ball on the ground, A is a ground pass to a mate.
- B: JUMP (reach high balls, the start of a bicycle kick).
- C: CROSS (VOORZET): lob it to a mate, from the ground or out of the
  air. Arrows pick the mate; no arrow = the mate closest to the goal.
- A ball on the ground near your feet sticks to you (dribble).
- A landing marker shows where a cross comes down.

Keeper:
- Arrows: move (stays near the goal).
- A: CATCH (with an arrow held: dive that way). Inside the area only,
  unless you are on 1 point. A catch is a WISSEL for the shooter.
- B: jump. C: PUNCH it away (play goes on; a rebound may be shot).
- Holding the ball: A rolls it out (arrows aim), or it goes by itself
  after a few seconds.

## Kids (select: SCHOT, KOP, KEEP, SNEL)
- KEVIN: wears his Oranje shirt in bed. Shoots from everywhere.
- SAMIRA: heads everything. Also things that aren't balls.
- GIJS: too big for the goal. Stands in it anyway.
- LIEKE: three years of bicycle kicks. Landed one. (Wider bicycle kick
  window.)

## Venues (1P ladder)
Each stage is one game of tienen against three local kids. You win the
stage when somebody else hits zero **and** you scored the pitch's target
in goal points (DOEL: 3, 4, 4): the big kids only respect you if you
score. You lose when you hit zero, or when somebody else does before you
reached the target (continue).
1. GALERIJFLAT, Saturday morning behind the flats: TIMMY (plays in
   his pyjamas), MO and JESSICA. Cars cross the pitch (AUTO!), a poodle
   chases the ball, a mum shouts from the balcony.
2. SCHOOLPLEIN, break time: DAAN, NOOR and BRAM from group 8. The
   janitor's bell, a fence with MEVROUW KNORR's garden behind it.
3. DE KOOI, the cage at dusk under the lamps: the big kids from the next
   street, with RICARDO (the rival who heckles), and the twins JAYDEN
   and JAYLA. The ice cream van.
Score: goal points you scored, saves, points you have left, billenbrand
hits, respect.

2P: two humans and two CPU kids on a venue picked in the menu. The game
ends when somebody hits zero. If a human lost, the other wins; if a CPU
lost, the human with more points left wins.

Attract mode: four CPU kids on a random venue.

## Whimsy
AUTO! (a car drives across the pitch; everybody steps aside; play
waits), the poodle (FIFI) that runs off with a loose ball, pigeons that
scatter, MEVROUW KNORR throws the ball back over the fence (or keeps
it), the ice cream van's tune, the mum on the balcony (ETEN!), a ball on
the balcony, heckles (HEB JE HEM?, KEEPER IS EEN GATENKAAS!), banners
(GOAL!, LAT!, PAAL!, GEVANGEN!, WISSEL!, GESTUITERD!, UIT!, OMHAAL!!!,
DOORGESCHOTEN!, BILLENBRAND!).

## CPU and balance
CPU field kids share the work: one fetches a loose ball, the others go
to scoring spots; the one with the ball crosses to the best placed mate;
the receiver times a volley, header, heel or bicycle kick, and a smart
kid picks the value that fits the keeper's points. CPU keepers read the
shot and move, jump, catch or punch, with reaction and error from skill.

`sim`: four CPU kids per venue (game length, keeper changes, goals by
kind, how points run out). `probe`: a human-like bot (decent, lazy,
greedy for bicycle kicks) with three CPU kids through the ladder.
Targets:
- A decent player wins stage 1 almost always (90%+) and stage 3 about a
  third of the time.
- Greed is punished: the bicycle-kick bot does worse than the decent one.
- A lazy keeper loses.
- A game takes about 1.5-3.5 minutes.

## As built (decisions)
- Why the target (DOEL): in tienen you only go in goal through your own
  mistake, so a 1P player who never touches the ball cannot lose. The
  target makes you play, and shooting is the risk.
- Crosses are lobs with a top of 36-58 px, high enough to head; a cross
  lands where the mate will be. The receiver lets a ball drop to the feet
  (a trap) unless they swing; a shot off your own touch does not count.
- A goal straight from a cross, or off your own touch, is AFGEKEURD: the
  keeper gets the ball back, nobody changes.
- A keeper's touch (a parry) counts as a pass through the air, so a
  rebound may be volleyed in. A keeper who punches the ball out puts the
  last field player in goal (the rule says field player).
- The keeper rolls the ball out (on the ground); a human keeper who waits
  three seconds rolls it out by itself.
- A dead ball nobody comes for (8 s) goes back to the keeper.
- After 5 minutes the mums call everybody in (ETEN!): the kid with the
  fewest points loses.
- The poodle takes any ball on the ground outside the keeper's area, even
  from somebody's feet; touch her and she drops it.
- Mevrouw Knorr only plays a part when the ball goes over the far fence at
  the school (OVER HET HEK!, a keeper change like any ball out).
