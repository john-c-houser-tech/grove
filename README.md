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

| Key   | Action                   |
|-------|--------------------------|
| Space | Plant a seed, or give up |
| ?     | Show the keys            |
| M     | Mute or unmute fog horns |
| Q     | Quit                     |
