# Goal of project

## Paralleization of multi agents in separate git Worktrees

The goal of this project is to be able to deploy from the terminal multiple coding agents all with the same prompt, running in parallel within their own git worktrees. This would be a cli application. For example, if we have a next.js website project, and we would like to run multiple spawns of [claude code](https://docs.anthropic.com/en/docs/claude-code/overview), [codex](https://openai.com/index/introducing-codex/), and [jules](https://jules.google/) in separate worktrees and allow them to complete their tasks independently. After they have completed we should be able to see the results of each agent's work in their respective worktrees. If it's running a local server, then it should spawn separate servers so that we can view the changes in realtime on the browser.

You can view an image of what the ideal web preview would look like in this folder: ./images/webpreview.jpg

# Implementation Details

The following blog post [Git Worktrees, Agents, and Tmux](https://www.skeptrune.com/posts/git-worktrees-agents-and-tmux/)
show proposes a solution using tmux. However, we should use native process and no containerization.

**Architecture Decision: Rust + ratatui**

- Use Rust for performance, safety, and single binary distribution
- Use ratatui for unified terminal UI instead of tmux
- Native process management with tokio for async agent orchestration
- Direct git worktree management without shell dependencies

- We should be able to choose how many spawns of the agents get launched.
- We should be able to choose which ones to use and how many of each.
- The cli application should present a great UX showing the responses from all agents in a unified interface.
- The cli application should allow for easy navigation and interaction with the agents.
- The application should allow for the user to answer any questions each agent may have as well as allow to instruct them to continue and complete the jobs.

**Multi-Provider Support:**

- Claude Code (Anthropic) - Desktop integration, local execution
- GPT/Codex (OpenAI) - API-based, flexible integration
- Jules (Google/Gemini) - GitHub-focused, cloud execution
- Unified agent abstraction layer for consistent interaction
- The cli application should present a great UX showing the responses from all agents in a unified interface.
- The cli application should allow for easy navigation and interaction with the agents.
- The application should allow for the user to answer any questions each agent may have as well as allow to instruct them to continue and complete the jobs.

## Technical Details

**Core Stack:**

- **Language**: Rust for performance, safety, and single binary distribution
- **UI**: ratatui for unified terminal interface (not tmux)
- **Async Runtime**: tokio for concurrent agent management
- **Git Operations**: git2-rs for worktree management
- **CLI Framework**: clap for command parsing
- **State Management**: SQLite or JSON for persistence

**Architecture:**

```
agentcrew/
├── src/
│   ├── main.rs
│   ├── cli/           # Command line interface (clap)
│   ├── agents/        # Agent provider abstractions
│   │   ├── claude.rs
│   │   ├── openai.rs
│   │   └── google.rs
│   ├── worktree/      # Git worktree management
│   ├── process/       # Agent process supervision
│   ├── ui/            # ratatui terminal interface
│   └── state/         # Session persistence
└── Cargo.toml
```

**Agent Communication:**

- Agents run in separate processes with stdin/stdout capture
- Questions queued in unified UI for user response
- Real-time status updates and log streaming
- Interactive response system with keyboard navigation

## Proposed commands from blog post using tmux.

Proposing a solution: agentcrew
To address these challenges head-on, the ideal developer experience (DX) would involve a lightweight CLI that wraps tmux, automating this complex orchestration. My co-founder Denzell and I felt these pain points acutely enough that we’ve begun developing such a tool, which we’re calling agentcrew. The core idea behind agentcrew is to abstract away the manual, repetitive tasks involved in managing multiple AI agent worktrees.

See some of the agentcrew commands we are thinking to implement below. Our goal is to make the workflow more seamless while providing a unified interface for agent interaction. We want to make sure that we feel at home using agentcrew alongside standard unix tools like xargs, grep, and awk.

**Core Commands:**

- `agentcrew init` - Initialize agentcrew in current project (creates .agentcrew config)
- `agentcrew deploy --agents claude:2,gpt:1,jules:1 --prompt "Implement feature X"` - Launch agents in separate worktrees
- `agentcrew status` - Display all active agents, progress, and worktree status
- `agentcrew tui` - Launch interactive terminal UI for real-time monitoring and interaction

**Agent Management:**

- `agentcrew list` - Show available agent types and their capabilities
- `agentcrew pause --agent claude-1` - Pause specific agent without terminating
- `agentcrew resume --agent claude-1` - Resume paused agent
- `agentcrew restart --agent gpt-1` - Restart failed or stuck agent
- `agentcrew dismiss --agent jules-1` - Terminate agent and cleanup resources

**Communication & Control:**

- `agentcrew brief "Focus on security best practices"` - Send instructions to all active agents
- `agentcrew respond --agent claude-1 "yes, use JWT"` - Answer agent questions from CLI
- `agentcrew broadcast --urgent "Stop and await new instructions"` - Priority message to all agents

**Worktree & Execution:**

- `agentcrew worktrees` - List all agent worktrees and their branches
- `agentcrew exec --all -- yarn dev` - Run commands across all agent worktrees
- `agentcrew exec --agent claude-1 -- npm test` - Run command in specific agent's worktree
- `agentcrew switch claude-1` - Switch to agent's worktree directory

**Progress & Results:**

- `agentcrew logs --agent gpt-1` - View specific agent's logs and output
- `agentcrew follow claude-1` - Follow agent's progress in real-time
- `agentcrew checkpoint --agent claude-1 --message "Auth system complete"` - Commit agent progress
- `agentcrew diff claude-1 gpt-1` - Compare code changes between agents
- `agentcrew harvest` - Collect and analyze results from all completed agents

**Session Management:**

- `agentcrew save session-name` - Save current multi-agent session
- `agentcrew load session-name` - Restore previous session
- `agentcrew history` - List previous sessions and their outcomes
- `agentcrew clean` - Clean up completed worktrees and temporary files

**Unified Interface Design (ratatui):**

```
┌─ Active Agents ──────────────────────────────────────────┐
│ claude-1 [auth] ████████░░ 80% ? "Use OAuth2?"           │
│ gpt-1    [tests]██████████ 100% ✓ Complete               │
│ jules-1  [docs] ██░░░░░░░░ 20% ↻ Working...              │
├─ Interaction Queue (2) ──────────────────────────────────┤
│ [claude-1]: Should I use OAuth2 or custom auth? [y/n/d]  │
│ [jules-1]: Which docs format? [md/rst/doc]               │
├─ Logs ───────────────────────────────────────────────────┤
│ 14:32 claude-1: Created auth middleware                  │
│ 14:31 gpt-1: All tests passing                          │
└─ Controls: Tab:Navigate Enter:Respond b:Brief q:Quit ──── ┘
```

This approach provides direct process management with a rich terminal interface, eliminating tmux dependencies while offering superior agent interaction capabilities.

<important>
I'm using this to learn RUST, so we need to ensure that the code is well-structured and follows best practices. We should also consider using Rust's built-in testing framework and linting tools to ensure code quality. Additionally, we should strive to write clean, modular code that is easy to understand and maintain. We should also follow Rust's naming conventions and use consistent formatting throughout the codebase. We should also use Rust's built-in documentation tools to generate documentation for the codebase. We should also use Rust's built-in profiling tools to optimize the codebase. We should also use Rust's built-in benchmarking tools to measure the performance of the codebase.

Most importantly, you should help implement this slowly and carefully teaching me along the way. This is very important, only write code in small chunks and explain each step. You have to ensure that I understand all the code implemented. This is the most important aspect of this project, so please take your time and ensure that I understand everything.
</important>

**IDE**
I USE ZED EDITOR.
