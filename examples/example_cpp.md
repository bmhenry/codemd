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
