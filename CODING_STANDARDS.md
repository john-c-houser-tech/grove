# Coding standards

Grove is a C++17 and Qt 6 Quick app. These rules keep it leak-free, quick to
draw and easy to read, and tools check as many of them as they can.

## Checks (tool-enforced)

| Command | What it checks |
|---------|----------------|
| `bin/lint` | Formatting (`.clang-format`); compiler warnings as errors (`-Wall -Wextra -Wpedantic -Wshadow -Werror`); `clang-tidy` (`.clang-tidy`); `qmllint` against the Grove module, with no warning allowed; every translation catalog complete. |
| `bin/test` | The unit tests, and the scene tests: the whole app drawn offscreen and driven by keys, where any QML warning fails. |
| `bin/check-leaks` | Both test programs under AddressSanitizer, LeakSanitizer and UndefinedBehaviorSanitizer, with no suppressions; then under Valgrind, when it is installed, with one suppression for a Qt read it misreports (`tests/valgrind.supp`); and a soak test that fails if the heap keeps rising. `--canary` plants a leak and kept memory, and passes only if every part catches them. |
| `bin/check` | All three. |

The pre-commit hook in `githooks/` runs `bin/lint` and `bin/test`; enable it
with `git config core.hooksPath githooks`. Run `bin/check` before a release,
and whenever a change touches ownership, threads, the QML engine, or anything
that lives as long as the app.

Don't argue with `clang-format`. A check is switched off only in
`.clang-tidy`, with the reason beside it. A single exemption in code
(`// NOLINT(check)` or `// qmllint disable`) names the check and says why on
the same or the line before.

## Memory and ownership

1. Every object has one owner: a Qt parent, a member, a `std::unique_ptr` or
   a container. `new` only with a parent or straight into a smart pointer;
   `delete` only where an API hands ownership back, as the scene graph does
   with nodes in `updatePaintNode`.
2. The objects QML uses belong to `App`. The singletons in `qmltypes.h` lend
   them to the engine with C++ ownership, and `App` makes its engine last and
   destroys it first, so QML never outlives what it shows.
3. Work on another thread gets everything it needs by value (the `Look`
   structs, a `PaintedLayer::Painting`) and never captures `this` or a
   `QObject`. Results come back on the GUI thread through a `QFutureWatcher`;
   an item destroyed meanwhile simply never sees its result.
4. Large buffers go as soon as they are used: `PaintedLayer` keeps no copy of
   a picture once the GPU has it, and `main()` fixes glibc's mmap threshold
   so freed pictures return to the system.
5. New state that lives as long as the app (a cache, a list that grows, a
   timer) gets a round in the soak test.

## C++

- C++17. Qt types where they meet Qt; standard containers and algorithms
  inside.
- Settings come in through `App::Options`. Only `App::fromEnvironment()` reads
  the environment.
- A problem the user should know about (an unreadable garden, no sound
  output) goes to the window through `Problem` (`problem.h`), whose words are
  translated each time they are shown.
- Each `PaintedLayer` subclass draws from a `Look` it copies to the drawing
  thread, and has a static `paint()` that tests can call directly.

## QML

- C++ is reached only through the Grove module: types marked `QML_ELEMENT`,
  and the singletons in `qmltypes.h`. No context properties, which no tool can
  check.
- `pragma ComponentBehavior: Bound` wherever there are delegates. Delegates
  declare what they use as `required property`, and reach other objects'
  properties through their ids.
- `let` and `const` in scripts, never `var`.
- No warnings at run time. The scene tests fail on any.

## Comments

- Every class, struct and QML component says what it is for.
- Every public function, property and signal says what its name does not:
  units and ranges, when a signal fires, who owns what, which thread it runs
  on. Overrides of Qt's functions and deleted copies and moves are exempt.
- Interface comments live in the header; the `.cpp` explains how and why.
- Comments explain why, and what is not obvious. They never restate the code.
  The landscape, light and sound are full of tuned numbers; each is explained
  where it is used, which is why `clang-tidy`'s magic-number check is off.

## Tests

- A bug fix comes with a test that fails without it. Check by undoing the fix.
- Unit tests cover classes; scene tests cover what a user does, through keys,
  in the real window.
- Nothing is shown on the desktop: tests run on the offscreen platform.
  `GROVE_SHOTS=dir build/scene_tests/scene_tests framesForReview` saves
  frames of several scenes for review. A visual change is checked by
  comparing them with the previous build's. `GROVE_README_SHOTS=dir` with
  `framesForReadme` makes the README's two pictures in the same way.
- Tests never touch the user's garden, theme, language, notifications or
  settings: `App::Options` points each at the test's own.

## Translations

Every string the user sees is a `tr("...")` or `qsTr("...")` literal, and is
in every catalog in `src/i18n/` (`bin/check-i18n`, part of `bin/lint`).
Numbers and dates go through the language's locale.

## Code smells (judgement calls)

Review changes against Fowler's code smells (*Refactoring*, ch. 3). These
are heuristics, never hard violations, and the rules above override them:
Mysterious Name, Duplicated Code, Feature Envy, Data Clumps, Primitive
Obsession, Repeated Switches, Shotgun Surgery, Divergent Change, Speculative
Generality, Message Chains, Middle Man, Refused Bequest.
