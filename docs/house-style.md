# House style

Every game on Helico Island must feel like it came out of the same 1992
studio (HELICO SOFT): a California Games / Ski or Die / Super Dodge Ball
era SNES cart. Whimsy is the point: every game has jokes in its art, its
rules and its text. Run `cargo run --release -- --shots shots` and look at
`shots/dodgeball/` as the reference.

## Screen and art
- 256x224 `Frame`, colors only through `rgb()` (15-bit SNES grid). No
  smooth gradients: bands of color, 2-color dithering (`dither_rect`),
  and `shade_rect` for shadows.
- Every sprite has the dark navy `OUTLINE` (`engine::ui::OUTLINE`).
  Characters are about 24x32 like the dodgeball players; use
  `engine::sprites::person()` with your own skeletons for people in new
  poses (on a bike, running uphill, hanging under a glider). Animals,
  vehicles and props are hand-drawn as string bitmaps with role letters,
  then palette-swapped, like `sprites.rs`.
- Backgrounds have depth: sky bands, far parallax layer, near layer, and
  small animated life (birds, flags, water, crowd, clouds).
- Big text uses `Frame::logo()` with `SUNSET` or a `grad6()` gradient,
  outline and `LOGO_SHADOW`. Small text uses the 5x7 font (`text_sh`,
  `text_center`), always with a shadow. The font is UPPER CASE only.
- Everything readable at 1x: nothing important smaller than the font.

## Screens and flow (in this order)
1. **Title**: the venue as backdrop, the game logo big at the top, a
   subtitle bar, blinking `PUSH START`, a scrolling ticker of jokes at the
   bottom and `(C) 1992 HELICO SOFT`. A/Start opens a small menu box:
   `1 PLAYER` / `2 PLAYERS` (and game-specific options if needed).
2. **Attract mode**: after about 15 s idle on the title, a CPU demo plays
   with a blinking `DEMO PLAY - PUSH START` bar. Any button returns to the
   title.
3. **Select**: pick your character, team, dog, bike or glider. Each choice
   shows stats as bars and a one-line joke bio.
4. **Intro card** before each stage or match (like the dodgeball versus
   card): venue name, who you face, a funny tagline.
5. **Play** with a HUD band at the top (score, time, meters), banners
   that slide in (`GO!`, `NICE!`, `WIPEOUT!`), popups for points.
   Start pauses (`PAUSE` logo + `PUSH START`).
6. **Results** after each stage: score tally that counts up with a blip
   per step, then a ranking.
7. **Champion / podium** screen with confetti and the win song; a
   **game over** or continue screen when you lose in 1P.

## Controls (fixed across games)
- P1: arrows, Z = A (main action), X = B (jump / hop / flare), C = C
  (second action), Enter = Start. P2: WASD, F, G, H, Tab.
- Esc: anywhere in the game goes to the game's own title; Esc on the title
  leaves the game (`wants_exit()`), back to the island.
- Every game has a 1P mode against the CPU and a 2P mode (versus at the
  same time, split screen with `Frame::copy_band`, or taking turns).
- Show the controls the first time they matter, in the game's own words
  (a tip box on the intro card is fine).

## Sound
- Use `engine::synth`: `Buf` for effects, `Song` for music. Each game has
  at least a title song, a play song and a win jingle, with its own
  character (key, tempo, drums), and 8-16 sound effects.
- Menu blip on cursor move, confirm on select, a whistle or horn for
  start/finish, a cheer for wins.

## Whimsy checklist (each game needs most of these)
- Named characters with silly bios; a rival who heckles.
- Animals that interfere (seagulls, geese, goats, cows, squirrels).
- Running jokes on the title ticker and loading cards.
- Secret or rare events (a UFO, a herring cart, a cow-shaped cloud).
- Visible failure is funny, not punishing (comedic crash, quick retry).

## Game structure
- 1P: a short ladder of 3 stages/opponents of rising difficulty, with
  continues, ending on the champion screen. A full run takes 5-10 min.
- Tune difficulty with a headless `sim` (CPU vs CPU) and, where it makes
  sense, a `probe` (scripted "human" bots: lazy, decent, exploit) so a
  decent player wins stage 1 almost always and stage 3 about a third of
  the time. Record the targets and results in the game's spec.
