# agentcrew

> Parallel AI agent orchestration in separate git worktrees

**agentcrew** is a Rust-powered CLI tool that deploys multiple AI coding agents (Claude, GPT, Jules) in parallel, each working in their own git worktree. Compare approaches, multiply productivity, and let AI agents tackle the same problem from different angles.

## 🚀 Features

- **Multi-Provider Support**: Claude Code, OpenAI GPT/Codex, and Google Jules/Gemini
- **Parallel Execution**: Agents work simultaneously in isolated git worktrees
- **Unified Interface**: Real-time monitoring and interaction through ratatui TUI
- **Smart Orchestration**: Native process management with tokio async runtime
- **Interactive Communication**: Queue and respond to agent questions seamlessly
- **Session Management**: Save, load, and replay multi-agent workflows
- **Development Integration**: Spawn dev servers, run tests, compare outputs

## 📦 Installation

### From Source (Recommended)

```bash
git clone https://github.com/your-org/agentcrew
cd agentcrew
cargo install --path .
```

### Pre-built Binaries

Download from [releases page](https://github.com/mkykode/agentcrew/releases)

### Package Managers

```bash
# Homebrew (macOS/Linux)
brew install agentcrew

# Cargo
cargo install agentcrew
```

## 🎯 Quick Start

1. **Initialize in your project**:

   ```bash
   cd your-project
   agentcrew init
   ```

2. **Deploy multiple agents**:

   ```bash
   agentcrew deploy --agents claude:2,gpt:1,jules:1 --prompt "Implement JWT authentication"
   ```

3. **Monitor with TUI**:

   ```bash
   agentcrew tui
   ```

4. **Collect results**:
   ```bash
   agentcrew harvest
   ```

## 💻 Usage Examples

### Basic Deployment

```bash
# Deploy agents with specific prompt
agentcrew deploy --agents claude:3,gpt:2 --prompt "Add dark mode to the UI"

# Check status
agentcrew status

# View specific agent logs
agentcrew logs --agent claude-1
```

### Interactive Development

```bash
# Start dev servers across all worktrees
agentcrew exec --all -- npm run dev

# Run tests in specific agent's worktree
agentcrew exec --agent claude-1 -- npm test

# Switch to agent's directory
agentcrew switch gpt-1
```

### Agent Communication

```bash
# Send instructions to all agents
agentcrew brief "Focus on accessibility compliance"

# Respond to agent questions
agentcrew respond --agent claude-1 "yes, use TypeScript"

# Urgent broadcast to all agents
agentcrew broadcast --urgent "Stop current work"
```

### Workflow Management

```bash
# Save current session
agentcrew save auth-implementation

# Load previous session
agentcrew load auth-implementation

# Compare agent outputs
agentcrew diff claude-1 gpt-1

# Clean up completed work
agentcrew clean
```

## 📋 Commands Reference

### Core Commands

| Command            | Description                             |
| ------------------ | --------------------------------------- |
| `agentcrew init`   | Initialize agentcrew in current project |
| `agentcrew deploy` | Launch agents in separate worktrees     |
| `agentcrew status` | Display all active agents and progress  |
| `agentcrew tui`    | Launch interactive terminal UI          |

### Agent Management

| Command                            | Description                 |
| ---------------------------------- | --------------------------- |
| `agentcrew list`                   | Show available agent types  |
| `agentcrew pause --agent <name>`   | Pause specific agent        |
| `agentcrew resume --agent <name>`  | Resume paused agent         |
| `agentcrew restart --agent <name>` | Restart failed agent        |
| `agentcrew dismiss --agent <name>` | Terminate agent and cleanup |

### Communication & Control

| Command                                       | Description                     |
| --------------------------------------------- | ------------------------------- |
| `agentcrew brief <message>`                   | Send instructions to all agents |
| `agentcrew respond --agent <name> <response>` | Answer agent questions          |
| `agentcrew broadcast --urgent <message>`      | Priority message to all         |

### Development & Execution

| Command                                      | Description                      |
| -------------------------------------------- | -------------------------------- |
| `agentcrew worktrees`                        | List all agent worktrees         |
| `agentcrew exec --all -- <command>`          | Run command across all worktrees |
| `agentcrew exec --agent <name> -- <command>` | Run command in specific worktree |
| `agentcrew switch <agent>`                   | Switch to agent's worktree       |

### Progress & Results

| Command                               | Description                     |
| ------------------------------------- | ------------------------------- |
| `agentcrew logs --agent <name>`       | View agent logs                 |
| `agentcrew follow <agent>`            | Follow agent progress real-time |
| `agentcrew checkpoint --agent <name>` | Commit agent progress           |
| `agentcrew diff <agent1> <agent2>`    | Compare changes between agents  |
| `agentcrew harvest`                   | Collect and analyze all results |

### Session Management

| Command                 | Description                  |
| ----------------------- | ---------------------------- |
| `agentcrew save <name>` | Save current session         |
| `agentcrew load <name>` | Restore previous session     |
| `agentcrew history`     | List previous sessions       |
| `agentcrew clean`       | Clean up completed worktrees |

## 🏗️ Architecture

```
agentcrew/
├── src/
│   ├── main.rs              # Entry point and CLI setup
│   ├── cli/                 # Command line interface (clap)
│   │   ├── mod.rs
│   │   ├── deploy.rs
│   │   ├── status.rs
│   │   └── tui.rs
│   ├── agents/              # Agent provider abstractions
│   │   ├── mod.rs
│   │   ├── claude.rs        # Claude Code integration
│   │   ├── openai.rs        # GPT/Codex API
│   │   ├── google.rs        # Jules/Gemini integration
│   │   └── traits.rs        # Common agent interface
│   ├── worktree/            # Git worktree management
│   │   ├── mod.rs
│   │   ├── manager.rs
│   │   └── cleanup.rs
│   ├── process/             # Agent process supervision
│   │   ├── mod.rs
│   │   ├── supervisor.rs
│   │   └── communication.rs
│   ├── ui/                  # ratatui terminal interface
│   │   ├── mod.rs
│   │   ├── app.rs
│   │   ├── widgets.rs
│   │   └── events.rs
│   └── state/               # Session persistence
│       ├── mod.rs
│       ├── session.rs
│       └── storage.rs
└── Cargo.toml
```

## ⚙️ Configuration

### Project Configuration (`.agentcrew/config.toml`)

```toml
[project]
name = "my-project"
default_agents = ["claude:1", "gpt:1"]

[agents.claude]
enabled = true
max_instances = 5

[agents.gpt]
enabled = true
api_key_env = "OPENAI_API_KEY"
model = "gpt-4"

[agents.jules]
enabled = true
github_integration = true

[worktree]
cleanup_on_exit = true
preserve_logs = true

[ui]
theme = "dark"
refresh_rate = 100
```

### Environment Variables

```bash
# OpenAI API Key
export OPENAI_API_KEY="your-api-key"

# Claude API Key
export ANTHROPIC_API_KEY="your-api-key"

# Google API Key
export GOOGLE_API_KEY="your-api-key"

# agentcrew Configuration
export AGENTCREW_CONFIG_DIR="$HOME/.config/agentcrew"
export AGENTCREW_LOG_LEVEL="info"
```

## 🎨 TUI Interface

The Terminal User Interface provides real-time monitoring and interaction:

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
│ 14:30 jules-1: Generated API documentation              │
└─ Controls: Tab:Navigate Enter:Respond b:Brief q:Quit ────┘
```

### Keyboard Shortcuts

- `Tab` / `Shift+Tab` - Navigate between agents
- `Enter` - Respond to selected agent question
- `b` - Send brief to all agents
- `p` - Pause/resume selected agent
- `d` - Dismiss selected agent
- `l` - View detailed logs
- `h` - Show help
- `q` - Quit TUI

## 🔧 Development

### Prerequisites

- Rust 1.70+
- Git 2.30+
- Access to AI provider APIs

### Building from Source

```bash
git clone https://github.com/your-org/agentcrew
cd agentcrew
cargo build --release
```

### Running Tests

```bash
cargo test
cargo test --integration-tests
```

### Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## 🎯 Use Cases

### Feature Development

Deploy multiple agents to implement the same feature with different approaches:

```bash
agentcrew deploy --agents claude:2,gpt:2 --prompt "Implement user authentication with password reset"
```

### Code Review & QA

Use agents to review and test each other's work:

```bash
agentcrew deploy --agents claude:1,gpt:1 --prompt "Review and test the auth implementation in main branch"
```

### Performance Optimization

Compare optimization strategies:

```bash
agentcrew deploy --agents claude:1,gpt:1,jules:1 --prompt "Optimize database queries in user service"
```

### Bug Investigation

Parallel debugging approaches:

```bash
agentcrew deploy --agents claude:3 --prompt "Debug memory leak in worker processes"
```

## 🛡️ Security & Privacy

- **Local Execution**: Agents run locally in isolated worktrees
- **API Key Management**: Secure environment variable handling
- **Code Isolation**: Each agent works in separate git branches
- **Audit Trail**: Complete logging of all agent actions
- **Cleanup**: Automatic cleanup of sensitive data

## 🔗 Integration

### CI/CD Pipeline

```yaml
# .github/workflows/agentcrew.yml
name: Multi-Agent Testing
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install agentcrew
        run: cargo install agentcrew
      - name: Run multi-agent tests
        run: |
          agentcrew deploy --agents claude:1,gpt:1 --prompt "Run full test suite"
          agentcrew harvest --format junit
```

### IDE Integration

- Zed (planned)

## 📊 Roadmap

- [x] Core CLI commands
- [x] Multi-provider agent support
- [x] ratatui TUI interface
- [ ] Web dashboard
- [ ] Plugin system
- [ ] Cloud execution support
- [ ] Team collaboration features
- [ ] Advanced analytics
- [ ] IDE integrations

## 🤝 Community

- **Discussions**: [GitHub Discussions](https://github.com/your-org/agentcrew/discussions)
- **Issues**: [Bug Reports & Feature Requests](https://github.com/your-org/agentcrew/issues)
- **Discord**: [agentcrew Community](https://discord.gg/agentcrew)
- **Documentation**: [Official Docs](https://agentcrew.dev)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by [Git Worktrees, Agents, and Tmux](https://www.skeptrune.com/posts/git-worktrees-agents-and-tmux/)
- Built with [ratatui](https://github.com/ratatui-org/ratatui) for terminal UI
- Powered by [tokio](https://tokio.rs/) for async runtime
- Git operations via [git2-rs](https://github.com/rust-lang/git2-rs)

---

**Made with ❤️ for developers who want to multiply their coding productivity with AI agents.**
