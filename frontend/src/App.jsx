
import { useEffect, useState } from "react";
import {
  Bot, ChevronRight, CheckCircle2, AlertCircle,
  RefreshCw, Play, ShieldCheck, Monitor, Sparkles,
  BrainCircuit, MousePointer2, ArrowUpRight, Loader2,Search
} from "lucide-react";
import "./App.css";

const API = "http://127.0.0.1:8000";

const pipeline = [
  { id: "input", title: "USER INSTRUCTION", detail: "Natural language", icon: Sparkles },
  { id: "snapshot", title: "UI SNAPSHOT", detail: "Windows interface", icon: Monitor },
  { id: "planner", title: "AI PLANNER", detail: "OpenRouter model", icon: BrainCircuit },
  { id: "approval", title: "APPROVAL", detail: "Human confirmation", icon: ShieldCheck },
  { id: "agent", title: "RUST AGENT", detail: "Windows execution", icon: MousePointer2 },
  { id: "result", title: "RESULT", detail: "Verified output", icon: CheckCircle2 },
];

export default function App() {
  const [apps, setApps] = useState(["Calculator"]);
  const [selectedApp, setSelectedApp] = useState("Calculator");
  const [selectedApps, setSelectedApps] = useState([]);
  const [mode, setMode] = useState("single");

  const [task, setTask] = useState("");
  const [planData, setPlanData] = useState(null);
  const [execution, setExecution] = useState(null);
  const [isMultiPlan, setIsMultiPlan] = useState(false);

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [backendOnline, setBackendOnline] = useState(false);
  const [activeTab, setActiveTab] = useState("Workspace");
  const [recentRuns, setRecentRuns] = useState([]);
  const [appSearch, setAppSearch] = useState("");

  async function loadApps() {
    try {
      const response = await fetch(`${API}/installed-apps`);

      if (!response.ok) {
        throw new Error("Unable to retrieve installed applications.");
      }

      const data = await response.json();

      const names = (data.applications || [])
        .map(item => item.name)
        .filter(Boolean);

      if (names.length > 0) {
        setApps(names);

        setSelectedApp(current =>
          names.includes(current) ? current : names[0]
        );

        setSelectedApps(current => {
          const valid = current.filter(name => names.includes(name));

          if (valid.length >= 2) return valid;

          return names.slice(0, Math.min(2, names.length));
        });
      } else {
        setApps([]);
        setSelectedApp("");
        setSelectedApps([]);
      }

      setBackendOnline(true);
      setError("");
    } catch (err) {
      setBackendOnline(false);
      setError(
        err.message ||
        "Backend connection unavailable. Check that FastAPI is running on port 8000."
      );
    }
  }

  useEffect(() => {
  let cancelled = false;

  async function initializeApps() {
    try {
      const response = await fetch(`${API}/installed-apps`);

      if (!response.ok) {
        throw new Error("Unable to retrieve installed applications.");
      }

      const data = await response.json();

      if (cancelled) return;

      const names = (data.applications || [])
        .map(item => item.name)
        .filter(Boolean);

      if (names.length > 0) {
        setApps(names);
        setSelectedApp(current =>
          names.includes(current) ? current : names[0]
        );
        setSelectedApps(current => {
          const valid = current.filter(name => names.includes(name));
          return valid.length >= 2
            ? valid
            : names.slice(0, Math.min(2, names.length));
        });
      } else {
        setApps([]);
        setSelectedApp("");
        setSelectedApps([]);
      }

      setBackendOnline(true);
      setError("");
    } catch (err) {
      if (cancelled) return;

      setBackendOnline(false);
      setError(
        err.message ||
        "Backend connection unavailable. Check that FastAPI is running on port 8000."
      );
    }
  }

  initializeApps();

  return () => {
    cancelled = true;
  };
}, []);

  function resetResults() {
    setPlanData(null);
    setExecution(null);
    setError("");
  }

  function toggleApp(appName) {
    setSelectedApps(current => {
      if (current.includes(appName)) {
        return current.filter(name => name !== appName);
      }

      if (current.length >= 5) {
        setError("You can select a maximum of 5 applications.");
        return current;
      }

      setError("");
      return [...current, appName];
    });

    setPlanData(null);
    setExecution(null);
  }

  async function generatePlan() {
    if (task.trim().length < 3) {
      setError("Enter an instruction with at least 3 characters.");
      return;
    }

    if (mode === "single" && !selectedApp) {
      setError("Select an application.");
      return;
    }

    if (mode === "multi" && selectedApps.length < 2) {
      setError("Select at least 2 applications for multi-app automation.");
      return;
    }

    setLoading(true);
    setError("");
    setPlanData(null);
    setExecution(null);

    try {
      const isMulti = mode === "multi";

      const endpoint = isMulti ? "/plan-multi" : "/plan";

      const body = isMulti
        ? {
            task: task.trim(),
            apps: selectedApps
          }
        : {
            task: task.trim(),
            app: selectedApp
          };

      const response = await fetch(`${API}${endpoint}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });

      const data = await response.json();

      if (!response.ok) {
        throw new Error(data.detail || "Unable to generate a plan.");
      }

      if (!data.success || !data.plan?.steps?.length) {
        throw new Error("The AI could not generate a valid action plan.");
      }

      setPlanData(data);
      setIsMultiPlan(isMulti);
      setActiveTab("Workspace");

    } catch (err) {
      setError(err.message || "Planning failed.");
    } finally {
      setLoading(false);
    }
  }

  async function executePlan() {
    if (!planData?.plan?.steps?.length) return;

    setLoading(true);
    setError("");
    setExecution(null);

    try {
      const body = isMultiPlan
        ? {
            snapshots: planData.snapshots,
            steps: planData.plan.steps,
            confirmed: true
          }
        : {
            app: selectedApp,
            snapshot: planData.snapshot,
            confirmed: true,
            steps: planData.plan.steps
          };

      const endpoint = isMultiPlan
        ? "/execute-multi"
        : "/execute";

      const response = await fetch(`${API}${endpoint}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });

      const data = await response.json();

      if (!response.ok) {
        throw new Error(data.detail || "Execution failed.");
      }

      setExecution(data);

      const actionResults = data.results || [];
      const successful = data.success === true;

      setRecentRuns(previous => [
        {
          task,
          app: isMultiPlan
            ? (planData.apps || selectedApps).join(", ")
            : selectedApp,
          count: actionResults.filter(item => item.success).length,
          time: new Date().toLocaleTimeString(),
          success: successful
        },
        ...previous,
      ]);

      setPlanData(null);

    } catch (err) {
      setError(err.message || "Execution failed.");
    } finally {
      setLoading(false);
    }
  }

  const steps = planData?.plan?.steps || [];
  const completed = execution?.success === true;

  const resultItems = execution?.results || [];
  const filteredApps = apps.filter(app =>
    app.toLowerCase().includes(appSearch.toLowerCase().trim())
  );

  const nodeState = (id) => {
    if (id === "input") return task ? "complete" : "idle";

    if (id === "snapshot" || id === "planner") {
      return planData || execution ? "complete" : "idle";
    }

    if (id === "approval") {
      return execution ? "complete" : planData ? "active" : "idle";
    }

    if (id === "agent" || id === "result") {
      return execution
        ? (completed ? "complete" : "active")
        : "idle";
    }

    return "idle";
  };

  return (
    <div className="enterprise-app">

      <header className="top-navigation">
        <div className="top-brand">
          <div className="brand-mark"><Bot size={19} /></div>
          <div>
            <span className="brand-overline">ENTERPRISE /</span>
            <strong>AutomaAI</strong>
          </div>
        </div>

        <nav className="navigation-links">
          {["Workspace", "Runs", "Applications", "Observability"].map(tab => (
            <button
              key={tab}
              className={activeTab === tab ? "nav-link selected" : "nav-link"}
              onClick={() => setActiveTab(tab)}
            >
              {tab}
            </button>
          ))}
        </nav>

        <div className="system-status">
          <span className={backendOnline ? "status-indicator" : "status-indicator offline"} />
          {backendOnline ? "System operational" : "Backend disconnected"}
        </div>
      </header>

      <main className="page-container">

        <section className="hero-section">
          <div className="hero-copy">
            <span className="micro-label">WINDOWS AUTOMATION PLATFORM</span>
            <h1>DESKTOP<br />AUTOMATION<br />AT WORK.</h1>
          </div>

          <div className="hero-description">
            <p>
              Control your Windows applications through natural language.
              Follow every step from instruction to verified execution.
            </p>
            <div className="hero-caption">
              <span /> AN INTELLIGENT DESKTOP WORKSPACE
            </div>
          </div>
        </section>

        <div className="workspace-strip">
          <span><strong>AUTOMAAI WORKSPACE</strong> · Live application</span>
          <span>
            {backendOnline
              ? "Connected to the existing agent API"
              : "Waiting for backend connection"}
          </span>
        </div>

        {activeTab === "Workspace" && (
          <>

            <section className="query-layout">

              <div className="query-main">

                <div className="section-kicker">ASK THE AGENT</div>
                <h2>What would you like to automate?</h2>

                <div className="query-box">

                  <div className="query-box-heading">
                    <ChevronRight size={14} />
                    INSTRUCTION / WINDOWS DESKTOP
                  </div>

                  <div className="mode-selector">
                    <button
                      className={mode === "single" ? "mode-option active" : "mode-option"}
                      onClick={() => {
                        setMode("single");
                        resetResults();
                      }}
                    >
                      <Monitor size={15} />
                      Single App
                    </button>

                    <button
                      className={mode === "multi" ? "mode-option active" : "mode-option"}
                      onClick={() => {
                        setMode("multi");
                        resetResults();
                      }}
                    >
                      <BrainCircuit size={15} />
                      Multi-App
                    </button>
                  </div>

                  {mode === "single" ? (

                    <select
                      className="app-select"
                      value={selectedApp}
                      onChange={e => {
                        setSelectedApp(e.target.value);
                        resetResults();
                      }}
                    >
                      {apps.map(app => (
                        <option key={app} value={app}>{app}</option>
                      ))}
                    </select>

                  ) : (

                    <div className="multi-app-selector">
                      <div className="multi-app-heading">
                        <strong>SELECT APPLICATIONS</strong>
                        <span>{selectedApps.length} / 5 selected</span>
                      </div>
                    <div className="app-search-box">
                      <Search size={16} />

                      <input
                        type="text"
                        placeholder="Search installed applications..."
                        value={appSearch}
                        onChange={e => setAppSearch(e.target.value)}
                      />

                      {appSearch && (
                        <button
                          type="button"
                          onClick={() => setAppSearch("")}
                        >
                          ×
                        </button>
                      )}
                    </div>
                      <div className="multi-app-list">
                        {filteredApps.map(app => (
                          <label
                            key={app}
                            className={
                              selectedApps.includes(app)
                                ? "multi-app-item selected"
                                : "multi-app-item"
                            }
                          >
                            <input
                              type="checkbox"
                              checked={selectedApps.includes(app)}
                              onChange={() => toggleApp(app)}
                            />
                            <Monitor size={15} />
                            <span>{app}</span>
                          </label>
                        ))}
                      </div>
                      {filteredApps.length === 0 && (
                        <p className="no-app-results">
                          No applications found.
                        </p>
                      )}

                      <p className="multi-app-hint">
                        Choose 2–5 applications. The agent will follow the
                        order described in your instruction.
                      </p>
                    </div>

                  )}

                  <textarea
                    value={task}
                    onChange={e => setTask(e.target.value.slice(0, 500))}
                    placeholder={
                      mode === "multi"
                        ? "Example: Search Google for Python tutorials in Chrome, then type Python tutorials in Notepad."
                        : "Describe a task for your Windows application..."
                    }
                    rows={3}
                  />

                  <div className="query-controls">
                    <div className="mode-options">
                      <span className="mode-chip">WINDOWS</span>
                      <span className="mode-chip">AI PLANNER</span>
                      <span className="mode-chip mode-active">
                        {mode === "multi" ? "MULTI-APP" : "SINGLE-APP"}
                      </span>
                    </div>

                    <div className="query-actions">
                      <button
                        className="clear-button"
                        onClick={() => {
                          setTask("");
                          resetResults();
                        }}
                      >
                        Clear
                      </button>

                      <button
                        className="run-button"
                        onClick={generatePlan}
                        disabled={
                          loading ||
                          !backendOnline ||
                          (mode === "multi" && selectedApps.length < 2)
                        }
                      >
                        {loading
                          ? <Loader2 size={15} className="spin" />
                          : <Play size={14} />}

                        {loading ? "PLANNING..." : "GENERATE PLAN"}

                        <ArrowUpRight size={14} />
                      </button>
                    </div>
                  </div>
                </div>

                <p className="query-hint">
                  Generate a plan, review the proposed actions, and approve
                  before allowing desktop control.
                </p>

              </div>

              <aside className="recent-panel">
                <div className="recent-heading">
                  <span>RECENT RUNS</span>
                  <span>{String(recentRuns.length).padStart(2, "0")}</span>
                </div>

                {recentRuns.length === 0 ? (
                  <p className="empty-recent">
                    Automation history from this session will appear here.
                  </p>
                ) : (
                  recentRuns.slice(0, 5).map((run, index) => (
                    <div className="recent-item" key={`${run.time}-${index}`}>
                      {run.success
                        ? <CheckCircle2 size={14} />
                        : <AlertCircle size={14} />}

                      <div>
                        <strong>{run.task}</strong>
                        <small>
                          {run.app} · {run.count} actions · {run.time}
                        </small>
                      </div>
                    </div>
                  ))
                )}
              </aside>

            </section>

            <section className="execution-section">

              <div className="execution-heading">
                <div>
                  <div className="section-kicker">SYSTEM / EXECUTION</div>
                  <h2>One instruction. A visible path.</h2>
                  <p>
                    Follow the instruction through planning, approval and
                    Windows execution.
                  </p>
                </div>

                <div className="legend">
                  <span><i className="legend-blue" /> Active</span>
                  <span><i className="legend-green" /> Complete</span>
                  <span><i className="legend-grey" /> Waiting</span>
                </div>
              </div>

              <div className="pipeline-card">
                <div className="pipeline-topline">
                  <span>
                    INSTRUCTION → SNAPSHOT → PLANNER → APPROVAL → AGENT → RESULT
                  </span>
                  <span>ROUTE · AUTOMATIC</span>
                </div>

                <div className="pipeline">
                  {pipeline.map((node, index) => {
                    const Icon = node.icon;
                    const state = nodeState(node.id);

                    return (
                      <div className="pipeline-unit" key={node.id}>
                        <div className={`pipeline-node ${state}`}>
                          <Icon size={18} />
                          <strong>{node.title}</strong>
                          <small>{node.detail}</small>
                        </div>

                        {index !== pipeline.length - 1 && (
                          <div className="pipeline-connector">
                            <ChevronRight size={15} />
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>

                <div className="pipeline-foot">
                  <span>Select a stage to understand the automation flow.</span>
                  <span>Node states reflect backend response data</span>
                </div>
              </div>

            </section>

            {error && (
              <div className="error-message">
                <AlertCircle size={17} /> {error}
              </div>
            )}

            {planData && (
              <section className="answer-section">

                <div className="answer-main">

                  <div className="section-kicker">
                    AI GENERATED {isMultiPlan ? "MULTI-APP " : ""}PLAN
                  </div>

                  <h2>Your plan is ready for review.</h2>

                  <p>{planData.plan?.response}</p>

                  <div className="plan-card">

                    <div className="plan-card-heading">
                      <span>PROPOSED ACTIONS</span>
                      <span>{steps.length} STEPS</span>
                    </div>

                    {steps.map((step, index) => (
                      <div className="action-row" key={`${step.app || selectedApp}-${index}`}>

                        <span className="action-index">
                          {String(index + 1).padStart(2, "0")}
                        </span>

                        <div>
                          {isMultiPlan && (
                            <small className="step-app-name">
                              {step.app}
                            </small>
                          )}

                          <strong>{step.action.toUpperCase()}</strong>

                          <p>
                            {step.reason || step.value || step.target_ref}
                          </p>
                        </div>

                        <code>
                          {step.target_ref || step.value || "—"}
                        </code>

                      </div>
                    ))}

                    <div className="approval-row">
                      <ShieldCheck size={17} />
                      Review the proposed actions before allowing desktop control.
                    </div>

                    <div className="approval-actions">

                      <button
                        className="cancel-button"
                        onClick={() => setPlanData(null)}
                        disabled={loading}
                      >
                        Cancel
                      </button>

                      <button
                        className="approve-button"
                        onClick={executePlan}
                        disabled={loading || !steps.length}
                      >
                        {loading
                          ? <Loader2 size={15} className="spin" />
                          : <Play size={14} />}

                        APPROVE & EXECUTE
                      </button>

                    </div>

                  </div>
                </div>

                <aside className="evidence-panel">
                  <div className="recent-heading">
                    <span>PLAN DETAILS</span>
                    <span>01</span>
                  </div>

                  <p>
                    <strong>Mode</strong><br />
                    {isMultiPlan ? "Multi-App" : "Single-App"}
                  </p>

                  <p>
                    <strong>Applications</strong><br />
                    {isMultiPlan
                      ? (planData.apps || selectedApps).join(", ")
                      : selectedApp}
                  </p>

                  <p>
                    <strong>AI model</strong><br />
                    {planData.model || "OpenRouter"}
                  </p>

                  <p>
                    <strong>Approval</strong><br />
                    Required before execution
                  </p>
                </aside>

              </section>
            )}

            {execution && (
              <section className="answer-section">

                <div className="answer-main">

                  <div className="section-kicker">EXECUTION RESULT</div>

                  <h2>
                    {completed
                      ? "Task completed successfully."
                      : "Execution stopped or partially completed."}
                  </h2>

                  <p>
                    {resultItems.filter(item => item.success).length}
                    {" of "}
                    {resultItems.length} actions completed successfully.
                  </p>

                  <div className="plan-card">

                    {resultItems.map(item => (
                      <div className="action-row" key={item.step}>

                        {item.success
                          ? <CheckCircle2 className="result-check" size={18} />
                          : <AlertCircle size={18} color="#d45c5c" />}

                        <div>
                          {isMultiPlan && (
                            <small className="step-app-name">
                              {item.app}
                            </small>
                          )}

                          <strong>
                            STEP {item.step} · {item.action.toUpperCase()}
                          </strong>

                          <p>
                            {item.output || item.error || "Action completed"}
                          </p>
                        </div>

                        <span className={item.success ? "done-label" : "failed-label"}>
                          {item.success ? "DONE" : "FAILED"}
                        </span>

                      </div>
                    ))}

                    {execution.failed_step && (
                      <div className="approval-row">
                        <AlertCircle size={17} />
                        Execution stopped at step {execution.failed_step}.
                      </div>
                    )}

                  </div>
                </div>

                <aside className="evidence-panel">
                  <div className="recent-heading">
                    <span>FINAL STATUS</span>
                    <span>01</span>
                  </div>

                  <p>
                    <strong>Applications</strong><br />
                    {isMultiPlan
                      ? (planData?.apps || selectedApps).join(", ")
                      : selectedApp}
                  </p>

                  <p>
                    <strong>Completed actions</strong><br />
                    {resultItems.filter(item => item.success).length}
                  </p>

                  <p>
                    <strong>Result</strong><br />
                    <span className={completed ? "result-success" : "failed-label"}>
                      {completed ? "Execution completed" : "Execution incomplete"}
                    </span>
                  </p>
                </aside>

              </section>
            )}

          </>
        )}

        {activeTab === "Runs" && (
          <section className="tab-page">
            <div className="section-kicker">WORKSPACE / HISTORY</div>
            <h2>Automation runs.</h2>
            <p>Your automation tasks from this session.</p>

            {recentRuns.length === 0 ? (
              <p className="empty-recent">
                No runs recorded yet. Start an automation from Workspace.
              </p>
            ) : (
              recentRuns.map((run, i) => (
                <div className="history-row" key={i}>
                  {run.success
                    ? <CheckCircle2 size={17} />
                    : <AlertCircle size={17} />}

                  <div>
                    <strong>{run.task}</strong>
                    <small>
                      {run.app} · {run.count} successful actions · {run.time}
                    </small>
                  </div>
                </div>
              ))
            )}
          </section>
        )}

        {activeTab === "Applications" && (
          <section className="tab-page">
            <div className="section-kicker">WINDOWS / APPLICATIONS</div>
            <h2>Available applications.</h2>
            <p>Applications and shortcuts registered in the Windows Start Menu.</p>

            <button className="refresh-apps" onClick={loadApps}>
              <RefreshCw size={14} /> Refresh applications
            </button>

            <div className="application-list">
              {apps.map(app => (
                <div className="application-item" key={app}>
                  <Monitor size={17} />
                  <strong>{app}</strong>
                  <span>Detected</span>
                </div>
              ))}
            </div>
          </section>
        )}

        {activeTab === "Observability" && (
          <section className="tab-page">
            <div className="section-kicker">SYSTEM / HEALTH</div>
            <h2>System observability.</h2>
            <p>Connection status of the desktop automation services.</p>

            <div className="health-card">
              <span className={backendOnline ? "status-indicator" : "status-indicator offline"} />
              <div>
                <strong>FastAPI backend</strong>
                <small>{backendOnline ? "Connected · Port 8000" : "Disconnected"}</small>
              </div>
            </div>

            <div className="health-card">
              <span className="status-indicator" />
              <div>
                <strong>Automation engine</strong>
                <small>Rust agent-cu · Windows</small>
              </div>
            </div>

            <div className="health-card">
              <span className="status-indicator" />
              <div>
                <strong>AI planner</strong>
                <small>OpenRouter · {planData?.model || "Configured"}</small>
              </div>
            </div>
          </section>
        )}

        <footer className="page-footer">
          <span>AUTOMAAI · WINDOWS DESKTOP AUTOMATION</span>
          <span>FASTAPI / OPENROUTER / RUST AGENT</span>
        </footer>

      </main>
    </div>
  );
}
