# CodeMD

This crate enables extra metadata in Markdown code blocks, which can be used to pull
and concatenate the code together for building & running. The primary purpose for this
is that code used in user-facing documentation can be pulled and tested to ensure it
works as expected. You might use it in CI when documentation updates, or when documented
libraries change.

Some languages (such as Rust) have this functionality built in as part of their existing
testing frameworks (e.g. `cargo test --doc`). However, most languages don't have this type
of framework. Furthermore, almost all existing solutions are intended to run a single block
of code all by itself, rather than allowing tutorial-style modification of code blocks which
may be interspersed with explanatory text.


## Example Scenario

Let's say you have a tutorial on how to write a simple "Hello, world!" program in C++.
You might have a Markdown file named `example_cpp.md` (check the `examples` folder)
that looks like this:

~~~markdown
# C++ Hello World Example

The "Hello, world!" program has, for whatever reason, become one of the first things
that coders learn to write in any new programming language.

In C++, you'll need to start by including the `iostream` header, which includes
functions for printing to `stdout`:

```cpp { "file": "example.cpp" }
#include <iostream>
```

Then, you'll need a main function. Included is the line that actually prints
the "Hello, world!" statement:

```cpp { "file": "example.cpp" }
int main() {
    std::cout << "Hello, world!" << std::endl;
    return 0;
}
```

What if you want to print from another function, instead of from main?
Let's write that function:

```cpp { "file": "example.cpp", "line": 1 }
void print_hw() {
    std::cout << "Hello, world!" << std::endl;
}
```

Then, we can modify our main function to instead look like this:
```cpp { "file": "example.cpp", "first": 4, "last": 7 }
int main() {
    print_hw();
    return 0;
}
```
~~~


You would run `codemd` on this file:

```sh
codemd -f examples/example_cpp.md
```

And you would get a single output file named `example.cpp` that looks like this:

```cpp
#include <iostream>
void print_hw() {
    std::cout << "Hello, world!" << std::endl;
}
int main() {
    print_hw();
    return 0;
}
```

At this point, the file will exist and you can take over with whatever build/run/test environment
you'd like to use.

