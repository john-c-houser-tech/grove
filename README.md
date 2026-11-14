# Grove

![Grove growing an autumn plant below the mountains](docs/grove.png)

A focus timer that grows a plant while you work. Press Space to plant a seed
in a furrow of the ploughed field.
Stay with it for 25 minutes and the plant blooms; give up and it wilts. Either
way it stays in your garden.

- **A view.** Distant peaks wear snow in winter, with rolling ridges and a
  hillside nearer in, each hazier with distance and lit like the rest.
- **Light follows the clock.** Dawn, noon, golden hour and night, with dusk
  warming toward your Omarchy accent. Day length follows the calendar for a
  mid-northern latitude. There is no location lookup.
- **Weather and seasons.** Each session rolls its own sky (clear, cloudy, rain,
  fog, and snow in winter only), weighted by the season, and it can change part-way through. Rain
  nudges growth along a little. Leaves are green, then autumn colours, then
  bare and frosted in winter. The weather is invented, never fetched.
- **Distant sounds.** Only while a plant is growing, and always faint. At
  night a train now and then approaches a crossing, with its horn, the bells
  and the rumble of the rails, one to three times in a session. You can see it
  too: a diesel with its headlight and ditch lights, and coaches with lit
  windows, come out from behind the hill on
  the right, run along the track across the valley and go in behind the hill
  on the left. The level crossing stands there day and night, and its lamps
  flash in turn with the bells. By day a
  small plane flies over now and then, crossing the sky while its engine drones
  very far off. All of it is synthesized and drawn at start-up, so nothing is
  shipped or downloaded. M mutes the sound (the picture stays).
- **A garden.** Every finished or wilted plant is kept, and G shows them all.
- **A nudge when it is done.** A desktop notification tells you the plant is
  fully grown, even if Grove is buried under other windows. Your shell's
  do-not-disturb silences it like any other. If there is no notification
  service, the window says so.

![The garden, with a plant for each session](docs/garden.png)

## Keys

| Key   | Action                   |
|-------|--------------------------|
| Space | Plant a seed, or give up |
| G     | Show or hide the garden  |
| M     | Mute or unmute sounds    |
| ?     | Show the keys            |
| Q     | Quit                     |

Giving up or quitting mid-session asks first, because the plant wilts.

## Install

On Arch or Omarchy: `bin/install` builds the package and installs it, and
Grove then appears in the launcher (Super + Space). To just run it:
`bin/build && build/grove`.

It needs `qt6-base`, `qt6-declarative`, `qt6-wayland` and `alsa-lib` to run,
and Rust (`cargo`), a C++ compiler and `pkgconf` to build: Grove is written in Rust, and
the bridge to Qt compiles C++ of its own.

## Languages

Grove follows the language chosen under Omarchy's Setup > Language, live.
Catalogs ship for Danish, German, Mexican Spanish, French, Brazilian
Portuguese and Japanese; anything else shows English. `bin/check-i18n` checks
that every catalog covers every string.

## Your data

Plants are kept in `~/.local/share/grove/garden.json`, written atomically. If
that file cannot be read, Grove leaves it exactly as it is, says so in the
window, and does not save over it.

## Trying things out

| Variable | Effect |
|----------|--------|
| `GROVE_SECONDS=30` | A short session. It is not saved to the garden, and sends no notification, unless `GROVE_GARDEN` is also set. |
| `GROVE_TIME=17:05` | Freeze the sky at that time. |
| `GROVE_DATE=2026-10-20` | Pin the date, and so the season. |
| `GROVE_WEATHER=fog` | Pin the weather: `clear`, `cloudy`, `rain`, `snow` or `fog`. |
| `GROVE_GARDEN=/path/garden.json` | Use another garden file. |
| `GROVE_SOUND_TEST=train` | Make a `train` or a `plane` happen a few seconds into the next session, whatever the hour. |
| `GROVE_SOUND_VOLUME=0.5` | Scale the volume from 0 to 1. |

## Development

Grove is Rust, with its window in QML; `cxx-qt` joins the two. `bin/test`
runs the unit tests and the scene tests, which drive the whole app offscreen.
`bin/lint` runs the static checks (formatting, compiler and clippy warnings,
qmllint, the translation catalogs, and `cargo audit` where it is installed)
and `bin/check-leaks` the memory checks (the Qt-facing tests and the scene
tests under Valgrind, and a soak test); `bin/check` runs everything.
`CODING_STANDARDS.md` says how the code is written and checked, and
`githooks/` holds a pre-commit hook (`git config core.hooksPath githooks`).
CI runs `bin/lint` and `bin/test` on every pull request; `bin/check-leaks` is
run by hand.

## License

MIT. The theme and language watching follow Omacut and Monologue by David
Heinemeier Hansson, also MIT.

Grove is built against the Qt on the machine it is built on, and uses Qt's
compiled QML, which reaches into Qt's internals. After an upgrade of Qt,
rebuild it with `bin/install`; the package installs a pacman hook that says
so when Qt is upgraded.
