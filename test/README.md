# Test Suite

This directory contains input programs and their expected MIPS output for
end-to-end testing of the mini-compiler.

## Structure

```
test/
├── in/              # Source programs (custom language)
│   ├── helloworld.txt
│   ├── fact.txt
│   ├── arrange.txt
│   ├── merge_sort.txt
│   └── contest.txt
└── out/             # Expected MIPS assembly / simulator output
    ├── helloworld.txt
    ├── fact.txt
    ├── arrange.txt
    ├── merge_sort.txt
    └── contest.txt
```

## Running the Tests

Use the batch shell runner (requires [MARS](http://courses.missouristate.edu/kenvollmar/mars/) simulator):

```bash
bash tools/test.sh
```

Or run the Rust integration tests directly:

```bash
cargo test
```
