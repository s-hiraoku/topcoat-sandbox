# Herdr Console

A Topcoat 0.5 dashboard that shows the coding agents currently running in [Herdr](https://herdr.dev).

## Main workflow

- read live agent data from `herdr agent list`
- see fleet totals and Working / Blocked / Idle states
- search by task, project, path, or agent type
- filter by Herdr agent status
- refresh the Topcoat shard without reloading the page
- focus an agent pane back in Herdr
- consume the same data as JSON from `GET /api/agents`

## Requirements

- Rust 1.95+
- Herdr 0.7.5+ with its local server running
- Topcoat CLI 0.5.0

```bash
cargo install topcoat-cli --version 0.5.0
herdr status
topcoat dev
```

Open <http://127.0.0.1:3000>.

If `herdr` is not on the server process's `PATH`, point to it explicitly:

```bash
HERDR_BIN=/absolute/path/to/herdr topcoat dev
```

## Topcoat features demonstrated

- server-rendered `view!` components and layouts
- typed app context holding the Herdr client
- signals for search, filters, and refresh state
- a server-rendered `#[shard]` backed by live CLI data
- form routes using Post/Redirect/Get
- typed JSON and CSS responses

This is a local operator tool. Before exposing it on a network, add authentication, authorization, and CSRF protection to the pane-focus route.
