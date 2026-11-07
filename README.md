# Grove

A focus timer that grows a plant while you work. Press Space to plant a seed;
a 25-minute session grows one procedurally drawn plant. Finish and it blooms.
Give up and it wilts.

Build and run: `bin/build && build/grove` (set `GROVE_SECONDS=30` for a short
session). `bin/test` runs the tests, `bin/install` installs the package.

`GROVE_TIME=17:05 build/grove` freezes the sky at that time. The light follows
the real clock otherwise, with day length taken from the calendar (no location).

| Key   | Action                   |
|-------|--------------------------|
| Space | Plant a seed, or give up |
| ?     | Show the keys            |
| Q     | Quit                     |
