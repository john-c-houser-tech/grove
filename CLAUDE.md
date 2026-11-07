# Grove

A focus timer for Omarchy that grows a plant while you work, in C++17 and Qt 6
Quick, built with qmake. See `README.md` for what it does and
`CODING_STANDARDS.md` for how the code is written and checked.

## Files

- `src/app.*` makes and wires every object and the QML engine; `main.cpp`
  only reads the environment into `App::Options`.
- `src/qmltypes.h` declares the Grove QML module's singletons.
- `src/layer.*` is the base for the landscape layers drawn in C++ (`hills`,
  `field`, `trackbed`); `src/terrain.h` holds what they share.
- `src/*.qml` is the window; `src/i18n/` the translation catalogs.
- `src/sources.pri` lists every source but `main.cpp`, for the app and both
  test programs.
- `tests/grove_tests.cpp` has the unit tests; `tests/scene_tests.cpp` drives
  the whole app offscreen.
- `pkgbuild/` packages it for Arch; `bin/install` builds and installs it.

## Commands

- `bin/build` builds `build/grove`.
- `bin/test` runs both test programs, offscreen.
- `bin/lint` runs every static check; `bin/check-leaks` the memory checks;
  `bin/check` everything. All must pass before work is called done.

## Rules for agents

- Never open Grove, or any window, on the user's desktop: it takes their
  focus while they type. Check the window with the scene tests, and look at
  it through `GROVE_SHOTS` frames (see `CODING_STANDARDS.md`).
- A change to how anything is drawn is compared, frame by frame, with the
  previous build's frames before it is committed.
- Run `bin/check` after any change to ownership, threads or the QML engine,
  and say what it reported.
- Commit only when asked. The repository is local, on `master`, with no
  remote.
