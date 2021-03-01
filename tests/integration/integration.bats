#!/usr/bin/env bats

setup() {
  cd $WORKSPACE
  unset CI
}

setup_dirs() {
  if [ -f "_Cargo.toml" ]; then
    mv _Cargo.toml Cargo.toml
  fi

  if [ -d "_git" ]; then
    mv _git .git && git reset --hard master
  fi
}

@test "it runs" {
  run semantic-rs
  [ "$status" -eq 1 ]
}

@test "fails without Cargo.toml" {
  cd empty-dir
  run semantic-rs
  [ "$status" -eq 1 ]
}

@test "fails on non-git directories" {
  cd not-a-repo
  setup_dirs

  run semantic-rs
  [ "$status" -eq 1 ]
}

@test "fails with broken Cargo.toml" {
  cd broken-cargo-toml
  setup_dirs

  run semantic-rs
  [ "$status" -eq 1 ]
}

@test "Initializes to v0.1.0" {
  cd initial-release
  setup_dirs

  semantic-rs --write=yes --release=no
  grep -q 'version = "0.1.0"' Cargo.toml
}

@test "Bumps to next minor" {
  cd next-minor
  setup_dirs

  grep -q 'version = "1.0.0"' Cargo.toml

  semantic-rs --write=yes --release=no

  grep -q 'version = "1.1.0"' Cargo.toml

  run git log --oneline --format=format:%s
  [ "${lines[0]}" = "Bump version to 1.1.0" ]
}

@test "No bump when no new commits" {
  cd no-bump
  setup_dirs

  grep -q 'version = "1.1.0"' Cargo.toml

  run semantic-rs --write=yes --release=no
  [ "$status" -eq 0 ]
  [[ "$output" =~ "No version bump. Nothing to do" ]]

  grep -q 'version = "1.1.0"' Cargo.toml
}

@test "No crash with malformed tags" {
  cd malformed-tag
  setup_dirs

  semantic-rs
}

@test "It creates a new tag with message" {
  cd new-tag
  setup_dirs

  run git tag -l
  [ "$output" = "v1.0.0" ]

  semantic-rs --write=yes --release=no

  run git tag -l
  [ "${lines[0]}" = "v1.0.0" ]
  [ "${lines[1]}" = "v1.1.0" ]
}

@test "Runs a dry-run by default" {
  cd dry-run
  setup_dirs

  semantic-rs
  grep -q 'version = "0.1.0"' Cargo.toml
}

@test "Runs in write-mode with CI=true" {
  cd write-mode
  setup_dirs

  CI=true semantic-rs --release=no
  grep -q 'version = "0.2.0"' Cargo.toml
}

@test "Runs in dry-run-mode with CI=true but dry-run forced" {
  cd write-mode-disabled
  setup_dirs

  CI=true semantic-rs --write=no
  grep -q 'version = "0.1.0"' Cargo.toml
}

@test "Respects Git environment variables" {
  cd env-vars
  setup_dirs

  export GIT_COMMITTER_NAME=semantic-rs
  export GIT_COMMITTER_EMAIL=semantic@rs

  semantic-rs --write=yes --release=no

  run git log --oneline --format=format:'%an %ae'
  [ "${lines[0]}" = "semantic-rs semantic@rs" ]

  unset GIT_AUTHOR_NAME
  unset GIT_COMMITTER_EMAIL
}

@test "Does not run when on wrong branch" {
  cd wrong-branch
  setup_dirs

  run semantic-rs --branch=hamster --release=no
  [[ "$output" =~ "Current branch is 'master', releases are only done from branch 'hamster'" ]]
}

@test "Does not fail when project has no remote" {
  cd has-no-remote
  setup_dirs

  CARGO_TOKEN=1234 run semantic-rs --write=yes

  [ "$status" -eq 0 ]
  [[ "$output" =~ "semantic-rs can't push changes or create a release on GitHub" ]]
}
