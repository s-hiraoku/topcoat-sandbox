use serde::Deserialize;
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::{Cx, app_context},
    router::{
        Router, RouterBuilderDiscoverExt,
        content::{Css, Form},
        error::{SeeOther, bad_request, see_other},
        layout, module_router, page, route,
    },
    runtime::{Event, shard},
    view::{component, view},
};

use crate::herdr::{Agent, AgentSummary, HerdrClient};

pub fn router(client: HerdrClient) -> Router {
    module_router!()
        .discover()
        .assets(AssetBundle::load().expect("run with `topcoat dev` so assets are bundled"))
        .app_context(client)
        .build()
}

fn client(cx: &Cx) -> &HerdrClient {
    app_context(cx)
}

#[layout]
async fn shell(slot: Result) -> Result {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <meta name="description" content="Live operations dashboard for Herdr coding agents">
                <title>"Agents | Herdr Console"</title>
                topcoat::dev::script()
                topcoat::runtime::script()
                <link rel="stylesheet" href="/styles.css">
            </head>
            <body>
                <a class="skip-link" href="#content">"Skip to agents"</a>
                <div class="app-shell">
                    <aside class="rail">
                        <a class="wordmark" href="/" aria-label="Herdr Console home">
                            <span class="wordmark-glyph" aria-hidden="true">"H"</span>
                            <span class="wordmark-text">"HERDR"</span>
                        </a>
                        <nav aria-label="Primary">
                            <a class="nav-item active" href="/" aria-current="page">
                                <span aria-hidden="true">"◫"</span><span>"Agents"</span>
                            </a>
                            <a class="nav-item" href="/api/agents">
                                <span aria-hidden="true">"⌁"</span><span>"API"</span>
                            </a>
                        </nav>
                        <div class="rail-foot">
                            <span class="connection-dot" aria-hidden="true"></span>
                            <span>"LOCAL"</span>
                        </div>
                    </aside>
                    <div class="surface">
                        <header class="topbar">
                            <div>
                                <span class="crumb-muted">"HERDR"</span>
                                <span class="crumb-separator" aria-hidden="true">"/"</span>
                                <strong>"AGENTS"</strong>
                            </div>
                            <div class="server-state">
                                <span class="connection-dot" aria-hidden="true"></span>
                                <span>"LOCAL CONSOLE"</span>
                            </div>
                        </header>
                        <main id="content" tabindex="-1">(slot?)</main>
                    </div>
                </div>
            </body>
        </html>
    }
}

#[page]
async fn dashboard() -> Result {
    view! {
        signal query = String::new();
        signal status = "all".to_owned();
        signal revision = 0.0;

        <section class="page-intro">
            <div>
                <p class="kicker">"LIVE RUNTIME"</p>
                <h1>"Agent fleet"</h1>
                <p class="intro-copy">"Every coding agent running in Herdr, in one operational view."</p>
            </div>
            <button
                class="sync-button"
                type="button"
                @click=$(|_event: Event| revision.increment())
            >
                <span aria-hidden="true">"↻"</span>
                "Sync now"
            </button>
        </section>

        <section class="control-strip" aria-label="Agent filters">
            <div class="search-field">
                <label for="agent-search">"Search agents"</label>
                <span aria-hidden="true">"⌕"</span>
                <input
                    id="agent-search"
                    type="search"
                    placeholder="Task, project, path…"
                    :value=$(query.get())
                    @input=$(|event: Event| query.set(event.target.value))
                >
            </div>
            <fieldset class="status-filter">
                <legend>"Filter by status"</legend>
                for (value, label) in [
                    ("all", "All"),
                    ("working", "Working"),
                    ("blocked", "Blocked"),
                    ("idle", "Idle"),
                    ("done", "Done"),
                ] {
                    <label>
                        <input
                            type="radio"
                            name="status"
                            value=(value)
                            checked=(value == "all")
                            @change=$(move |_event: Event| status.set(value.to_owned()))
                        >
                        <span>(label)</span>
                    </label>
                }
            </fieldset>
        </section>

        agent_board(
            query: $(query.get()),
            status: $(status.get()),
            revision: $(revision.get())
        )
    }
}

#[shard]
async fn agent_board(cx: &Cx, query: String, status: String, revision: f64) -> Result {
    let _ = revision;

    match client(cx).agents().await {
        Ok(agents) => {
            let summary = AgentSummary::from_agents(&agents);
            let visible: Vec<Agent> = agents
                .into_iter()
                .filter(|agent| agent.matches(&query, &status))
                .collect();

            view! {
                summary_bar(summary: summary)

                <section class="agent-section" aria-labelledby="agent-heading">
                    <div class="section-heading">
                        <div>
                            <p class="kicker">"SESSION SNAPSHOT"</p>
                            <h2 id="agent-heading">"Running agents"</h2>
                        </div>
                        <p aria-live="polite">
                            (visible.len()) " of " (summary.total) " shown"
                        </p>
                    </div>

                    if visible.is_empty() {
                        empty_state()
                    } else {
                        <ul class="agent-grid" role="list">
                            for agent in visible {
                                agent_card(agent: agent)
                            }
                        </ul>
                    }
                </section>
            }
        }
        Err(error) => view! {
            <section class="error-state" role="alert">
                <span class="error-code">"CONNECTION / 01"</span>
                <h2>"Herdr is unavailable"</h2>
                <p>(error.to_string())</p>
                <p>"Start the Herdr server, then use Sync now."</p>
            </section>
        },
    }
}

#[component]
async fn summary_bar(summary: AgentSummary) -> Result {
    view! {
        <section class="summary-bar" aria-label="Fleet summary">
            metric(label: "Total", value: summary.total, class_name: "metric-total")
            metric(label: "Working", value: summary.working, class_name: "metric-working")
            metric(label: "Blocked", value: summary.blocked, class_name: "metric-blocked")
            metric(label: "Idle", value: summary.idle, class_name: "metric-idle")
        </section>
    }
}

#[component]
async fn metric(label: &str, value: usize, class_name: &str) -> Result {
    view! {
        <article class=(("metric ", class_name))>
            <span class="metric-label">(label)</span>
            <strong>(value)</strong>
            <span class="metric-mark" aria-hidden="true"></span>
        </article>
    }
}

#[component]
async fn agent_card(agent: Agent) -> Result {
    let status = agent.status;
    let card_class = if agent.focused {
        "agent-card is-focused"
    } else {
        "agent-card"
    };

    view! {
        <li class=(card_class)>
            <article>
                <div class="card-topline">
                    <div class=(("agent-avatar status-", status.as_str())) aria-hidden="true">
                        (agent.kind.chars().next().unwrap_or('A').to_ascii_uppercase())
                    </div>
                    <div class="agent-identity">
                        <h3>(&agent.kind)</h3>
                        <span>((&agent.workspace_id, " · ", &agent.pane_id))</span>
                    </div>
                    <span class=(("status-pill status-", status.as_str()))>
                        <span aria-hidden="true"></span>(status.label())
                    </span>
                </div>

                <div class="task-block">
                    <span>"CURRENT TASK"</span>
                    <p>(agent.task())</p>
                </div>

                <dl class="agent-meta">
                    <div><dt>"PROJECT"</dt><dd>(agent.project())</dd></div>
                    <div><dt>"REVISION"</dt><dd>(agent.revision)</dd></div>
                    <div class="path-row"><dt>"PATH"</dt><dd title=(agent.cwd.clone())>(agent.cwd)</dd></div>
                </dl>

                <div class="card-footer">
                    if agent.focused {
                        <span class="focused-label"><span aria-hidden="true">"●"</span>" Focused pane"</span>
                    } else {
                        <span class="focused-label muted">"Background pane"</span>
                    }
                    <form action="/focus" method="post">
                        <input type="hidden" name="pane_id" value=(agent.pane_id.clone())>
                        <button type="submit">"Focus in Herdr"<span aria-hidden="true">"↗"</span></button>
                    </form>
                </div>
            </article>
        </li>
    }
}

#[component]
async fn empty_state() -> Result {
    view! {
        <div class="empty-state">
            <span aria-hidden="true">"⌁"</span>
            <h3>"No matching agents"</h3>
            <p>"Change the status filter or search another project."</p>
        </div>
    }
}

#[derive(Deserialize)]
struct FocusAgent {
    pane_id: String,
}

#[route(POST "/focus")]
async fn focus_agent(cx: &Cx, Form(input): Form<FocusAgent>) -> Result<SeeOther> {
    client(cx)
        .focus(&input.pane_id)
        .await
        .map_err(|error| bad_request(error.to_string()))?;
    Ok(see_other("/"))
}

#[route(GET "/api/agents")]
async fn agents_api(cx: &Cx) -> Result<topcoat::router::content::Json<Vec<Agent>>> {
    let agents = client(cx)
        .agents()
        .await
        .map_err(|error| bad_request(error.to_string()))?;
    Ok(topcoat::router::content::Json(agents))
}

#[route(GET "/styles.css")]
async fn styles() -> Result<Css<&'static str>> {
    Ok(Css(include_str!("styles.css")))
}
