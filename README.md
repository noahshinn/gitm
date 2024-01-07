# Search

## Requirements

[`rust`](https://www.rust-lang.org/tools/install)

## To run

Install

```bash
git clone https://github.com/noahshinn/gitm && cd ./gitm
```

Build

```bash
cargo build
```

Run a search over commits

```bash
./target/release/gitm --query "your query" --api-key <your OpenAI key>
```

Other options

- `--issues-only`: run a search over issues only
- `--issues-too`: run a joint search over commits and issues
- `--include-code-patches`: use code diffs from commit patches during search
