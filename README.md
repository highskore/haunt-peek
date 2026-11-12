# Peek

Hold ⌥ over a Haunt row to peek at that agent's terminal.

Rest the pointer on a row in Haunt's sidebar and hold ⌥: a small card floats over Haunt with the last lines of that tab's screen, titled with the agent's name. Let go and the card is gone, with nothing switched. It also closes when you leave the row, press any key or Haunt loses focus. Haunt only watches ⌥: every key still reaches the terminal.

```
web
$ npm test session
 PASS  session.test.ts (14)
 RUNS  refresh.test.ts
▍
```

Screen text is untrusted: escape sequences and hidden characters are dropped, and lines are cut to 80 characters.

## Settings

```toml
[plugins."dev.haunt.peek"]
lines = 8           # lines in the card, 1 to 50
unverified = false  # also follow events without the tab's nonce
```

## Capabilities

| capability | why |
|---|---|
| `events` (session start, prompt, subagent, tool, permission request, question, stop) | each agent's name, for the card's title; no event text is kept |
| `keys` | ⌥ held over a row (one plugin at a time) |
| `terminal_read` | the last lines of that agent's screen, only while you hold ⌥ over its row |
| `panel` | the card |

## Build and test

```sh
cargo build --release --target wasm32-wasip2   # or ./build.sh
cargo test                                     # unit tests
haunt plugin install .
```

`e2e/peek.rs` runs the built plugin against a real Haunt core: copy it into a Haunt checkout's `cli/tests/` and run `HAUNT_PLUGINS_DIR=<this repo's parent> cargo test -p haunt-cli --features plugins --test peek`.

Needs Haunt with plugin API 0.5 (`api = "0.5"`) and its `keys.hover-hold` and `terminal` imports.

## Licence

MIT, see `LICENSE`.
