# Coding standards

Grove is a Rust and Qt 6 Quick app: the window is QML, and Rust reaches Qt
through `cxx-qt`. These rules keep it leak-free, quick to draw and easy to
read, and tools check as many of them as they can.

## Checks (tool-enforced)

| Command | What it checks |
|---------|----------------|
| `bin/lint` | Formatting (`rustfmt.toml`); the compiler's and clippy's warnings as errors; `qmllint` against the Grove module, with no warning allowed; every translation catalog complete; `cargo audit` over the packages in `Cargo.lock`, where it is installed (CI always has it). |
| `bin/test` | The unit tests, and the scene tests: the whole app drawn offscreen and driven by keys, where anything Qt writes to standard error fails the test. |
| `bin/check-leaks` | The unit tests of `src/qml/` and the scene tests under Valgrind, with one suppression for a Qt read it misreports (`tests/valgrind.supp`); and a soak test that fails if the heap keeps rising. `--canary` plants a leak and kept memory, and passes only if both parts catch them. |
| `bin/check` | All three. |

The pre-commit hook in `githooks/` runs `bin/lint` and `bin/test`; enable it
with `git config core.hooksPath githooks`. Run `bin/check` before a release,
and whenever a change touches unsafe code, threads, the QML engine, or
anything that lives as long as the app.

Don't argue with `rustfmt`. A clippy lint is switched off only where it
cannot be met, with the reason beside it: `#[expect(lint, reason = "...")]`
on the item, so the exemption goes when its cause does, or on the crate
where the code a macro generates is what trips it (`src/lib.rs`). A single exemption in
QML (`// qmllint disable`) names the check and says why on the same or the
line before.

Valgrind is the only memory checker here. The compilers' sanitizers, which
the C++ Grove also ran, are in Rust's nightly toolchain only.

## No C++

Nothing here is written in C++: no `.cpp` or `.h` file, and no shim. `cxx-qt`
generates and compiles C++ of its own during the build, which is why a C++
compiler is still needed to build Grove.

What `cxx-qt-lib` has no Rust for is declared in a bridge and called as Qt
wrote it:

1. A Qt class Grove subclasses is the `#[base]` of a QObject, its virtual
   functions overridden with `#[cxx_override]` (`translator.rs`, `layers.rs`).
2. A Qt object is made with `cxx-qt-lib`'s own templates, bound as functions:
   `#[namespace = "rust::cxxqtlib1"] #[cxx_name = "make_unique"]` for one on
   the heap, `"construct"` for a value.
3. A static function is bound with `#[Self = "Type"]` on a type declared in
   the same bridge.
4. A type Qt declares inside a class (`QQuickItem::Flag`) is written by hand
   as a Rust type with `unsafe impl cxx::ExternType`, and named in the bridge
   with `#[namespace = "QQuickItem"]` (`layer.rs`).
5. A class `cxx-qt-lib` also binds gets its own Rust name here and its
   methods a `q_` prefix, so the generated symbols differ. Where
   `cxx-qt-lib` already owns that class through a `UniquePtr` (`QPainter`),
   it is owned through `cxx-qt-lib`'s and cast for the calls declared here,
   since a second `UniquePtr` of it collides too (`paint.rs`).

## Memory, ownership and unsafe code

1. Unsafe code lives in `src/qml/`, and every `unsafe` block says, in a
   `// SAFETY:` comment, why it is sound; clippy fails one that does not.
   The rules above `src/qml/` are safe Rust that knows nothing of Qt. The one
   exception is `src/lib.rs`, which starts the application and makes one
   call into the allocator.
2. There is one `App` (`app.rs`), on the GUI thread, behind `qml/shared.rs`.
   The QML singletons own nothing of it: each reads what it shows from there
   and is told what changed. `qml::start` makes it before a window is loaded.
3. A change to the app returns the `Event`s it brought about, and the
   singletons queue their signals for them. They are queued, never emitted
   from inside the change, since the change may have come from the very
   object being told.
4. Work on another thread gets everything it needs by value (the `Look`
   structs, a `Painting`), and comes back to the GUI thread through the
   object's `qt_thread()`; an object destroyed meanwhile simply never sees
   the result. A thread stops when what it serves is gone.
5. Nothing of the audio system or the session bus is touched from the GUI
   thread: the player and the notifier each work on a thread of their own.
6. The services that live in QML singletons (following the theme and the
   language, the notification, the sounds) are started by Rust at launch, so
   no edit to the QML can switch one off. A scene test guards it.
7. Large buffers go as soon as they are used: a layer keeps no copy of a
   picture once the GPU has it, and `run()` fixes glibc's mmap threshold so
   freed pictures return to the system.
8. New state that lives as long as the app (a cache, a list that grows, a
   thread) gets a round in the soak test.

## Rust

- The rules of a thing are in a file that knows nothing of Qt, with its unit
  tests beside it; a thin QObject in `src/qml/` shows it to QML.
- Settings come in through `Options`. Only `Options::from_environment()`
  reads the environment.
- A problem the user should know about (an unreadable garden, no sound
  output) is an enum in the rules, put into words by the singleton that shows
  it, through `tr()`, each time it is shown, so it follows a change of
  language.
- Each layer draws from a `Look` it copies to the drawing thread, and has a
  `paint()` that tests call directly.
- Dependencies stay on their current release. An API that changed is
  learned, not avoided by pinning an older version.

## QML

- Rust is reached only through the Grove module: the items in `layers.rs`
  and the singletons in `src/qml/`. No context properties, which no tool can
  check.
- `pragma ComponentBehavior: Bound` wherever there are delegates. Delegates
  declare what they use as `required property`, and reach other objects'
  properties through their ids.
- `let` and `const` in scripts, never `var`. Some older scripts still use
  `var`; they are changed as they are touched.
- Numbers and dates are formatted with `Qt.locale(Translator.locale)`: Rust
  cannot set Qt's default locale.
- No warnings at run time. The scene tests fail on any.

## Comments

- Every module, struct, enum, trait and QML component says what it is for.
- Every public function, property and signal says what its name does not:
  units and ranges, when a signal fires, who owns what, which thread it runs
  on. Overrides of Qt's functions and plain accessors are exempt.
- Comments explain why, and what is not obvious. They never restate the
  code. The landscape, light and sound are full of tuned numbers; each is
  explained where it is used.

## Tests

- A bug fix comes with a test that fails without it. Check by undoing the fix.
- Unit tests cover the rules; scene tests cover what a user does, through
  keys, in the real window.
- Nothing is shown on the desktop: the scene-test program always runs on the
  offscreen platform, whatever the session sets, so every command below is
  headless as written. `GROVE_SHOTS=dir bin/test --test scene --
  frames_for_review` saves frames of several scenes for review; a visual
  change is checked by comparing them with the previous build's.
  `GROVE_README_SHOTS=dir bin/test --test scene -- frames_for_readme` makes
  the README's two pictures the same way.
- `GROVE_LAYER_SHOTS=dir bin/test frames_for_comparison` saves each layer's
  own picture, for comparing a layer pixel for pixel across two builds.
- Tests never touch the user's garden, theme, language, notifications or
  settings: each test's `Options` points every one of them at the test's own,
  and plays sounds to no one (`silent`). `Options::default()` is not enough
  on its own: it names the desktop's notification service and opens the
  real audio device.
- A scene test that asks the window the same question many times asks it
  through `Scene`, which makes each script once: the QML engine keeps what it
  compiles for every script made.

## Translations

Every string the user sees is a `qsTr("...")` literal in QML or a `tr("...")`
literal in Rust, and is in every catalog in `src/i18n/` (`bin/check-i18n`,
part of `bin/lint`). Numbers and dates go through the language's locale.

## Code smells (judgement calls)

Review changes against Fowler's code smells (*Refactoring*, ch. 3). These
are heuristics, never hard violations, and the rules above override them:
Mysterious Name, Duplicated Code, Feature Envy, Data Clumps, Primitive
Obsession, Repeated Switches, Shotgun Surgery, Divergent Change, Speculative
Generality, Message Chains, Middle Man, Refused Bequest.
