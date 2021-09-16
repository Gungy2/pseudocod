# Pseudocode
Simple interpreter for the [pseudocode langauge](https://www.pbinfo.ro/articole/23972/limbajul-pseudocod "Language Specification") used to teach programming in Romanian high schools. It is made entirely in Rust, using the Nom parser.

## Usage
In order to run the interpreter with a file, execute the command:
```bash
cargo run -- filename
```

## Example of pseudocode
```
scrie 'Introduceti n:'
citeste n
daca n = 0 atunci
  scrie 'fib(', n, ') = ', 0
altfel
  daca n = 1 atunci
    scrie 'fib(', n, ') = ', 1
  altfel
    x <- 0
    y <- 1
    pentru i <- 0, n - 2 executa
      tmp <- x
      x <- y
      y <- tmp + x
    scrie 'fib(', n, ') = ', y
```

## Using it for teaching
If you are a teacher that wants to use the tool, feel free to contact me and I will do my best to add any features that you feel are missing!
