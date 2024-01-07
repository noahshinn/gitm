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

- `--issues-only`: Run a search over issues only.
- `--issues-too`: Run a joint search over commits and issues.
- `--include-code-patches`: Use code diffs from commit patches during search.
- `--disable-classifications`: By default, your query will be parsed for potential filters (expressed in natural language). Set this flag to disable these checks.
