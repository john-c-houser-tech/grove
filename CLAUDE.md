# Grove

A focus timer for Omarchy that grows a plant while you work, in Rust and Qt 6
Quick, built with Cargo. The window is QML; Rust reaches Qt through `cxx-qt`.
See `README.md` for what it does and `CODING_STANDARDS.md` for how the code is
written and checked.

## Files

- `src/*.rs` holds the rules, which know nothing of Qt: the session, weather,
  light, garden, theme, language, sounds. `src/app.rs` ties them together and
  says what changed; `src/options.rs` is the only reader of the environment.
- `src/qml/` holds the Grove QML module: a thin QObject over each of those
  (`backend.rs`, `garden.rs`...), all sharing the one `App` in `shared.rs`.
  `mod.rs` starts the app and loads the window.
- `src/qml/paint.rs` is Qt's painter as the landscape layers use it,
  `layer.rs` what the layers share, `layers.rs` their QML items, and
  `hills.rs`, `field.rs` and `trackbed.rs` their drawings. `src/terrain.rs`
  holds the noise they share.
- `src/*.qml` is the window; `src/i18n/` the translation catalogs.
- `src/lib.rs` is everything but `main`; `src/main.rs` only calls it.
- `build.rs` lists the QML files and every Rust file with a bridge in it. A
  bridge left out still compiles, but QML never hears of its types.
- Unit tests sit beside the code they test; `tests/scene.rs` drives the whole
  app offscreen, through `src/qml/testing.rs`.
- `pkgbuild/` packages it for Arch; `bin/install` builds and installs it.
- `githooks/pre-commit` runs `bin/lint` and `bin/test` before each commit;
  `.github/workflows/ci.yml` runs them on every PR.

## Commands

- `bin/build` builds `build/grove`.
- `bin/test` runs the unit tests and the scene tests, offscreen.
- `bin/lint` runs every static check; `bin/check-leaks` the memory checks;
  `bin/check` everything. All must pass before work is called done.
- `git config core.hooksPath githooks` enables the pre-commit hook (lint and
  tests), once per clone. Until then nothing runs at commit time, so run
  `bin/lint` and `bin/test` yourself. The hook checks the working tree, so
  unstaged edits count.

## Rules for agents

- Never open Grove, or any window, on the user's desktop: it takes their
  focus while they type. Check the window with the scene tests, and look at
  it through `GROVE_SHOTS` frames (see `CODING_STANDARDS.md`).
- A change to how anything is drawn is compared, frame by frame, with the
  previous build's frames before it is committed.
- No C++ is written here: no `.cpp` or `.h` file, and no shim. What
  `cxx-qt` has no Rust for is declared in a bridge and called as Qt wrote it
  (`CODING_STANDARDS.md` says how).
- Dependencies stay on their current release. An API that changed is learned,
  not avoided by pinning an older version.
- Run `bin/check` after any change to unsafe code, threads or the QML engine,
  and say what it reported.
- Commit only when asked.

## Git

The repository is https://github.com/john-c-houser-tech/grove. `main` is
John's: never commit or push to it, and never force-push. `dev` is the
integration branch.

1. Before any new work: `git checkout dev && git pull --ff-only`.
2. Branch from dev: `git checkout -b <feature>`. Check `git branch --show-current`
   before every commit.
3. Commit there, push the branch, and open a PR into dev: `gh pr create --base dev`.
4. Wait for CI to pass: `gh pr checks <n> --watch`. CI
   (`.github/workflows/ci.yml`) runs `bin/lint` and `bin/test` in an Arch
   Linux container that installs from Omarchy's stable mirror, the one the
   desktop uses, so Qt and Rust match a desktop that is up to date. If CI
   fails lint where the desktop passes, compare the versions its Packages step
   prints with `pacman -Q`. CI does not run `bin/check-leaks`: run it (or
   `bin/check`) yourself whenever `CODING_STANDARDS.md` asks for it. dev's
   ruleset doesn't require CI, but main's requires the `CI result` check, so a
   red PR merged into dev blocks the next merge to main. Keep that job's name.
5. Before review requests, run independent agent reviews of the PR and fix
   what they find. Then request a Copilot review (`gh pr edit <n> --add-reviewer @copilot`)
   when Copilot credits are available, and address its comments with
   follow-up commits.
6. John merges to main. Wait for him.
