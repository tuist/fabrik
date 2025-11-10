# Makefile for Fabrik C API example

# Detect platform
UNAME_S := $(shell uname -s)

# Set library extension based on platform
ifeq ($(UNAME_S),Darwin)
    LIB_EXT = dylib
    LDFLAGS_EXTRA = -Wl,-rpath,../../target/release
else ifeq ($(UNAME_S),Linux)
    LIB_EXT = so
    LDFLAGS_EXTRA = -Wl,-rpath,../../target/release
else
    LIB_EXT = dll
    LDFLAGS_EXTRA =
endif

CC = gcc
CFLAGS = -Wall -Wextra -I../../include
LDFLAGS = -L../../target/release -lfabrik $(LDFLAGS_EXTRA)

.PHONY: all clean run build-lib

all: build-lib example

# Build the Rust library first
build-lib:
	@echo "Building Fabrik library..."
	cd ../.. && cargo build --release --lib

# Build the example
example: example.c
	@echo "Building C example..."
	$(CC) $(CFLAGS) -o example example.c $(LDFLAGS)
	@echo "✓ Build complete"

# Run the example
run: example
	@echo "Running example..."
	@./example

# Clean build artifacts
clean:
	rm -f example
	@echo "✓ Cleaned"

help:
	@echo "Fabrik C API Example Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all        - Build library and example (default)"
	@echo "  build-lib  - Build Fabrik shared library"
	@echo "  example    - Build C example only"
	@echo "  run        - Run the example"
	@echo "  clean      - Remove build artifacts"
	@echo "  help       - Show this help"
