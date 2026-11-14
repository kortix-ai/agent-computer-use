# Agent Computer Use — Frontend

A React-based web interface for interacting with the Agent Computer Use desktop automation system.

The frontend provides a workspace for describing tasks in natural language, reviewing AI-generated execution plans, approving actions, and viewing execution results.

## Features

- **Workspace:** Submit natural-language desktop automation tasks.
- **Application Management:** Discover and select installed applications.
- **Multi-Application Workflows:** Plan tasks involving multiple applications.
- **AI Planning:** Review generated automation steps before execution.
- **Human Approval:** Require confirmation before executing planned actions.
- **Execution Results:** View completed actions and execution status.
- **Run History:** Review recent automation activity.
- **Observability:** Inspect workflow progress.

## Technology Stack

- React
- Vite
- JavaScript
- Lucide React
- ESLint

## Prerequisites

- Node.js
- npm
- A compatible Agent Computer Use FastAPI backend

## Installation

Navigate to the frontend directory:

```bash
cd frontend
```

Install dependencies:

```bash
npm install
```

## Running the Development Server

```bash
npm run dev
```

Open the local URL displayed by Vite, usually:

```text
http://localhost:5173
```

## Available Commands

| Command | Description |
|---|---|
| `npm run dev` | Start development server |
| `npm run build` | Create production build |
| `npm run lint` | Run ESLint |
| `npm run preview` | Preview production build |

## Backend Integration

The frontend communicates with a FastAPI backend configured at:

```text
http://127.0.0.1:8000
```

The frontend expects backend endpoints for:

- Installed application discovery
- Single-application planning
- Multi-application planning
- Single-application execution
- Multi-application execution

**Important:** This frontend contribution does not include the backend implementation. A compatible backend must be running for application discovery, planning, and execution features to work.

## Project Structure

```text
frontend/
├── public/
│   ├── favicon.svg
│   └── icons.svg
├── src/
│   ├── assets/
│   ├── App.jsx
│   ├── App.css
│   ├── index.css
│   └── main.jsx
├── index.html
├── package.json
├── eslint.config.js
└── vite.config.js
```

## Development Notes

The frontend is maintained separately from the backend and Rust execution components.

Changes to the frontend should preserve compatibility with the backend API contract.
