# aicommit-rs 🔨 (in progress)

Using Google Gemini to generate git comments.

Read `examples/aicommit-template` to understand what exactly is generated.

## Installation

### Build from source

Clone the repository and build the project:

```bash
cargo build --path .
```

Add `~/.cargo/bin/` into your PATH.

### Download pre-built binaries

You can download pre-built binaries in releases section.

If you use [mise](https://mise.jdx.dev/), add following lines to your `~/.config/mise/config.toml`:

```toml

[tools]
"cargo:your_github_name/aicommit-rs" = "latest"
```

Copy `aicommit.toml` and `aicommit-template` into your home directory:

```bash
cp examples/aicommit-template ~/.aicommit-template
```

Or through environment variables:

```bash
AI_COMMIT_API_KEY=your-api-key aicommit-rs
```

Follow instructions to get your API key from [Google Gemini](https://ai.google.dev/gemini-api/docs/quickstart)

## CLI flags and arguments

Read [docs/usage.md](docs/usage.md) for more information.

## Examples

### [lazygit](https://github.com/jesseduffield/lazygit)

Add following _as a menu_ custom command in your `~/.config/lazygit/config.yml`:

```yaml
customCommands:
  - key: "<c-a>" # Ctrl + a
    description: "pick AI commit"
    command: 'git commit -m "{{.Form.Msg}}"'
    context: "files"
    prompts:
      - type: "menuFromCommand"
        title: "ai Commits"
        key: "Msg"
        command: "aicommit-rs"
        filter: '^(?P<number>\d+)\.\s(?P<message>.+)$'
        valueFormat: "{{ .message }}"
        labelFormat: "{{ .number }}: {{ .message | green }}"
```
