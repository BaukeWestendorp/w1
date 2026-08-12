# w1

`w1` is a small text-processing macro language written in Rust.

## Example

```html
@macro card($title, $content) {{
<div class="card">
    <h2>$title</h2>
    <div class="card-body">
        $content
    </div>
</div>
}}

@call card({{Welcome}}, {{<p>Lorem ipsum dolor, sit amet consectetur.</p>}})
```

Evaluates to:

```html
<div class="card">
    <h2>Welcome</h2>
    <div class="card-body">
        <p>Lorem ipsum dolor, sit amet consectetur.</p>
    </div>
</div>
```

## CLI

```text
Usage: w1 [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Path to the source file to parse. If omitted or "-", reads from stdin

Options:
  -o, --output <FILE>  Path to the output file. If omitted, writes to stdout
      --show-tokens    Skip building AST evaluation and output the parsed tokens for debugging
      --show-ast       Skip evaluation and output the parsed AST for debugging
  -h, --help           Print help
  -V, --version        Print version
```

### Write output to a file

```sh
w1 input.w1 --output output.txt
```

## License

See `LICENSE`.
