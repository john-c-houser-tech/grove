# Grove

A focus timer that grows a plant while you work. Press Space to plant a seed;
a 25-minute session grows one procedurally drawn plant. Finish and it blooms.
Give up and it wilts.

Build and run: `bin/build && build/grove` (set `GROVE_SECONDS=30` for a short
session). `bin/test` runs the tests, `bin/install` installs the package.

`GROVE_TIME=17:05 build/grove` freezes the sky at that time. The light follows
the real clock otherwise, with day length taken from the calendar (no location).

Each session rolls its own weather (clear, cloudy, rain or fog), weighted by
the season, and it can change part-way through. Rain nudges growth along a
little. Leaves follow the season: green, then autumn colours, then bare and
frosted in winter. In fog, a deep horn sounds now and then, with a quieter
answer from farther out; it is synthesized at start-up, never fetched.
`GROVE_WEATHER=fog` and `GROVE_DATE=2026-10-20` pin the weather and the date.

Every finished or wilted plant is kept in `~/.local/share/grove/garden.json`
(written atomically; a file that cannot be read is left untouched and the
window says so). Quitting mid-session wilts the plant. Press G to see them all.
`GROVE_GARDEN=/path/garden.json` uses another file, and a session shortened
with `GROVE_SECONDS` is not saved unless you set it.

| Key   | Action                   |
|-------|--------------------------|
| Space | Plant a seed, or give up |
| ?     | Show the keys            |
| G     | Show or hide the garden  |
| M     | Mute or unmute fog horns |
| Q     | Quit                     |
