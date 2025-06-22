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
"ubi:your_github_name/aicommit-rs" = "v0.0.7"
```

Copy `aicommit.toml` and `aicommit-template` into your home directory:

```bash
cp examples/aicommit.toml ~/.aicommit.toml
cp examples/aicommit-template ~/.aicommit-template
```

Replace `~/.aicommit.toml` with your own configuration file.

```bash
gemini_api_key = ""
gemini_api_url = "https://generativelanguage.googleapis.com/v1beta/openai"
model_name = "gemini-2.0-flash"

```

Follow instructions to get your API key from [Google Gemini](<https://ai.google.dev/gemini-api/docs/quickstart>)

## CLI flags and arguments

Read [docs/usage.md](docs/usage.md) for more information.

## Examples

### [lazygit](https://github.com/jesseduffield/lazygit)

Add following *as a menu* custom command in your `~/.config/lazygit/config.yml`:

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
