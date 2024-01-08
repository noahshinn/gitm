# gitm

A search tool for git and GitHub. This is a follow-up of [gcl](https://github.com/noahshinn/gcl) with more features. `gitm` allows you to search commits and issues by similar natural language strings in messages or similar code in code patches. `gitm` will also parse your queries so that you can express intent in ways beyond the ability of git's regex parsers. For example, the queries `only get johns commits on the new classification head` or `i want the change that added jq execution from at least a week ago` will be parsed to fulfill your author and recency filter requests.

## Requirements

[`rust`](https://www.rust-lang.org/tools/install)

## To install

Clone this repository

```bash
git clone https://github.com/noahshinn/gitm && cd ./gitm
```

Initialize and update submodules

```bash
git submodule update --init --recursive
```

Build and move the binary to /usr/local/bin

```bash
sudo make
```

Set your OpenAI API key (or pass it to the binary with the `--api-key` flag)

```bash
export OPENAI_API_KEY="<your key>"
```

Run a search over your commits

```bash
gitm "<your query>"
```

Other options

- `--issues-only`: Run a search over issues only.
- `--issues-too`: Run a joint search over commits and issues.
- `--include-code-patches`: Use code diffs from commit patches during search. This is useful in situations in which commit messages are ambigious (such as "Update").
- `--disable-classifications`: By default, your query will be parsed for potential filters (expressed in natural language). Set this flag to disable these checks.
- `--api-key`: An alternative OpenAI API key (other than OPENAI_API_KEY env var) to use.
- `--all`: Run a search over all commits in the current working repository. By default, `gitm` searches through the last two months of data if the current working repository contains more than 1000 commits.
- `--help`: See all of the options.
