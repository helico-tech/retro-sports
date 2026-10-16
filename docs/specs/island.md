# Helico Island (the game selector)

Module `src/hub/`. The first thing you see: an island map in the same
16-bit style, where every game is a venue.

- Top: the RETRO SPORTS logo and a subtitle bar.
- Middle: the island in the sea (animated waves, clouds, gulls, a blimp
  trailing a HELICO SOFT banner). Each game is a small animated diorama:
  the beach court (dodgeball), a park with a dog (Disc Dogs), the village
  of Helicodam with a canal, a fence's boat and a BOA chasing a bike thief
  (Stalen Zenuwen), a trapveldje behind the village where a kid heads a
  cross past a diving keeper (Tienen), a polder with a windmill and canal
  (Tegenwind), a mountain with a trail (Trail Mix), a cliff with a glider
  (Hang Time). A dotted path links them.
- A little `person()` walks the path between venues: left/right (or
  up/down) moves to the next venue; A or Start enters it with a jump and
  an iris wipe.
- Bottom: an info panel with the game's logo, its sport and a blurb.
- Esc on the island quits (desktop only; in the browser it does nothing).
- Its own island song and a few sounds (step, select, enter).
