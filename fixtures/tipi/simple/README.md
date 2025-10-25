# Simple Tipi Project

A minimal C project using Tipi with CMake Remote Execution (cmake-re).

## Project Structure

- `main.c` - Simple Hello World program
- `CMakeLists.txt` - Standard CMake build configuration
- `.tipi/deps` - Tipi configuration (`"u": true` tells tipi to use the CMakeLists.txt)

## About Tipi and CMake RE

Tipi provides CMake Remote Execution (cmake-re), which extends standard CMake projects with:
- **Build caching** - Automatic caching of build artifacts
- **Remote execution** - Build on cloud infrastructure
- **Hermetic builds** - Containerized, reproducible builds

A tipi/cmake-re project is a standard CMake project with environment descriptions for reproducibility.

## Building

### Prerequisites for cmake-re

cmake-re requires write access to `/usr/local/share/.tipi`. You have several options:

**Option 1: Fix permissions (simplest)**
```bash
sudo mkdir -p /usr/local/share/.tipi
sudo chown $(whoami) /usr/local/share/.tipi
```

**Option 2: Use symbolic link (recommended for local development)**
```bash
# Create tipi home in your project directory
mkdir -p .tipi-home
sudo mkdir -p /usr/local/share
sudo ln -s "$(pwd)/.tipi-home" /usr/local/share/.tipi
```

**Option 3: Use bind mount (for systems without permission to /usr/local)**
```bash
mkdir -p ~/.tipi-home
sudo mkdir -p /usr/local/share/.tipi
sudo mount --bind ~/.tipi-home /usr/local/share/.tipi
```

### Local Build (non-hermetic, with caching)

```bash
# Build using cmake-re (requires cmake-re binary from tipi installation)
cmake-re --host .

# Or use standard CMake (no special permissions needed)
mkdir -p build
cd build
cmake ..
cmake --build .
./main
```

### With Docker (hermetic build)

```bash
# Requires Docker installed
cmake-re .
```

### Remote Execution

```bash
# Build on tipi.build cloud (requires account)
cmake-re --remote .
```

## Installation

Tipi includes both `tipi` and `cmake-re` binaries.

### Option 1: Using mise with ubi backend (recommended for this project)

```bash
# Install both tipi and cmake-re via mise (automatic via postinstall hook)
mise install
```

**Note**: cmake-re requires both `tipi` and `cmake-re` binaries to be present. The mise configuration includes a `postinstall` hook that automatically downloads and installs both binaries after installation.

### Option 2: Install directly from tipi.build

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/tipi-build/cli/master/install/install_for_macos_linux.sh)"
```

This installs both binaries system-wide.

## Learn More

- [Tipi Documentation](https://tipi.build/documentation)
- [CMake RE Getting Started](https://tipi.build/documentation/0000-getting-started-cmake)
- [Example Project](https://github.com/tipi-build/get-started)
