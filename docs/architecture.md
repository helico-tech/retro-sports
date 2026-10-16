# Architecture

RETRO SPORTS is one crate: a library with a shared engine and one module per
game, plus a thin binary (`src/main.rs`) that owns the window, keyboard and
speakers through macroquad. The same binary builds for the browser
(`wasm32-unknown-unknown`, `web/`), hosted on Cloudflare at
retro-sports.helico.app (ADR 0002, ADR 0003).

## Layout
```
src/
  main.rs        window, keys, sound loading, command line
  lib.rs
  cart.rs        Game trait, Cart entry, Bank/Sound
  console.rs     Console: the hub or one running game; sound cues per bank
  engine/        shared by every game (no game rules in here)
    gfx.rs       Frame (256x224 software framebuffer), primitives, 5x7 font, logo()
    sprites.rs   skeleton people: poses, 6 heads, color roles, outline, person()
    synth.rs     chiptune synth: Buf (tone/hiss/hat), Song + render_song, wav()
    input.rs     Pad (4 dirs, a, b, c, start), edge(), any()
    rng.rs       Rng (xorshift, deterministic)
    ui.rs        OUTLINE, LOGO_SHADOW, SUNSET, grad6/grad7
    png.rs       save_png, save_sheet, write_rgb (headless screenshots)
  hub/           Helico Island: the game selector
  games/
    mod.rs       CARTS: every game, in selector order
    dodgeball/   DODGE OR DIE!
    discdogs/    DISC DOGS!     (frisbee)
    tegenwind/   TEGENWIND!     (Dutch cycling)
    trailmix/    TRAIL MIX!     (trail running)
    hangtime/    HANG TIME!     (hang gliding)
```

## How a game plugs in
A game is a struct that implements `cart::Game`:
`update(pads, esc)` at 60 Hz, `draw(&mut Frame)`, `take_sfx()` and
`music()` as indexes into its own sound bank, and `wants_exit()` when the
player presses Esc on its title. Its module exports a `pub const CART: Cart`
with its id, title, blurb, constructor, `sounds()` (all its sfx and songs as
samples), `shots(dir)` and optional headless tools (`sim`, `probe`). Adding
it to `games::CARTS` is the only shared line it touches. `tests/carts.rs`
checks the contract for every game.

The console gives each game bank `i + 1` (the hub is bank 0). The window
layer generates and loads a bank the first time it is needed, so start-up
stays fast in the browser.

## Key rules
- Games are deterministic given a seed and the pad inputs. Nothing in the
  library reads the clock, the keyboard or the window.
- Everything visible goes through `Frame`, so headless screenshots show
  exactly what the window shows.
- The engine has no game rules. A game that needs something new first
  builds it in its own module; move it to the engine only when a second
  game needs it, and never change existing engine behaviour silently
  (dodgeball's screenshots must stay byte-identical).
- Every game follows [house-style.md](house-style.md): same screens, same
  controls, same look.

## Dodgeball notes
- `game` — `Match`: players, ball, rules, CPU AI. World coordinates: x
  across (centre line at 128), y depth (0 = far line, 72 = near line), z
  height. `game::proj` maps them to the screen.
- `court` — `Scenery`: static background drawn once per match, plus the
  animated layer (crowd, water, snow, lights).
- `render` — draws a `Match`; `app` — the scene flow; `audio` — its sounds;
  `headless` — `shots`, `sim`, `probe`.

Balance targets (tune with `--sim dodgeball` and `--probe dodgeball`):
- CPU vs CPU: rounds 45-70 s, time-ups under 12%, supers 10-15% of
  throws, every crew 47-53% (with 1000 matches).
- Probe "Decent" bot across the ladder: about 80-100 / 55 / 30% wins.
- Exploits: super spam under 40% at stage 3, stalling no better than
  playing.

Each other game keeps its own notes and targets in `docs/specs/<game>.md`.
