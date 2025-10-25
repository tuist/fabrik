# Simple Tipi Project

A minimal C project using the Tipi build system.

## Project Structure

- `main.c` - Simple Hello World program
- `.tipi/deps` - Tipi dependency configuration (empty for this simple example)

## Building

```bash
# Build the project
tipi build .

# Run the executable
tipi run .
```

## About Tipi

Tipi is a build system and compiler-as-a-service for C/C++ projects that works by convention. It automatically discovers source files and builds them without requiring extensive configuration like CMakeLists.txt or Makefiles.

Learn more at https://tipi.build
