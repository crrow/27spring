@book:
    mdbook serve docs

# Calculate code
@cloc:
    cloc . --exclude-dir=vendor,docs,tests,examples,build,scripts,tools,target

@fmt:
    cargo +nightly fmt
    taplo format
    taplo format --check
    hawkeye format

alias c := check
@check:
    cargo check --all --all-features --all-targets

tui:
    cargo run -- tui