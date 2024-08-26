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

Lines are 0 indexed.


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

```cpp { "append": { "file": "example.cpp" } }
#include <iostream>
```

Then, you'll need a main function. Included is the line that actually prints
the "Hello, world!" statement:

```cpp { "append": { "file": "example.cpp" } }
int main() {
    std::cout << "Hello, world!" << std::endl;
    return 0;
}
```

What if you want to print from another function, instead of from main?
Let's write that function:

```cpp { "insert": { "file": "example.cpp", "line": 1 } }
void print_hw() {
    std::cout << "Hello, world!" << std::endl;
}
```

Then, we can modify our main function to instead look like this:
```cpp { "diff": { "file": "example.cpp", "first": 4, "last": 7 } }
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


## Markdown Extended Syntax

`codemd` uses an extended syntax at the end of code block lines which doesn't conflict with the
existing Markdown syntax (in fact, it'll be ignored by Markdown parsers unless they specifically
include additional metadata parsing as well). This extra metadata allows code blocks to be concatenated
into files for external testing.

The syntax is as follows:

~~~
```<language> { "<command>": { "<key>": "<value>", ... } }
```
~~~


### Commands

#### `append`

Appends the code block to the end of a file

Keys:
- `file`: _Optional_ The name of the file to append to
- `removals`: _Optional_ A list of removal objects which allow extra lines to be removed from the code block after the append is done


#### `insert`

Inserts the code block into a file at a specific line

Keys:
- `file`: _Optional_ The name of the file to insert into
- `line`: _Required_ The line number to insert the code block at
- `removals`: _Optional_ A list of removal objects which allow extra lines to be removed from the code block after the insert is done


#### `diff`

Replaces a range of lines in a file with the code block. `diff` will first remove the specified range of lines, and
will then insert the code block at the `first` line.

Keys:
- `file`: _Optional_ The name of the file to replace in
- `first`: _Required_ The first pre-existing line number to remove, and the line number to insert the code block at
- `last`: _Required_ The last pre-existing line number to remove
- `removals`: _Optional_ A list of removal objects which allow extra lines to be removed from the code block after the diff is done


#### Removals

Removals are not a command, but are used in the other commands. They allow additional sets of lines to be
removed from the code file **after** the command has been executed.

This is useful in cases where a command may add or change a chunk of code, but a separate section of code should be
removed due to the change.

**NOTE:** It is up to YOU to tell the user what code is being removed, especially in the case that the Markdown doc is
a tutorial!

Keys:
- `first`: _Required_ The first line number to remove
- `last`: _Required_ The last line number to remove

