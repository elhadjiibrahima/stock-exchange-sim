#!/usr/bin/env zsh

# Build the binary
go build -o stock

# Run own examples
./stock examples/zen
# Pause before continuing
echo "Press Enter to continue to the given examples..."
read
./stock examples/macguffin
# Pause before continuing
echo "Press Enter to continue to the given examples..."
read
./stock examples/matryushka 0.002
echo ""
# Pause before continuing
echo "Press Enter to continue to the given examples..."
read

# Run the given examples
./stock examples/simple 1
# Pause before continuing
echo "Press Enter to continue to the given examples..."
read
./stock examples/build 10
# Pause before continuing
echo "Press Enter to continue to the given examples..."
read
./stock examples/seller 10
# Pause before continuing
echo "Press Enter to continue to the given examples..."
read
./stock examples/fertilizer 0.001
# Pause before continuing
echo "Press Enter to continue to the given examples..."
read
./stock examples/fertilizer 0.0003
echo ""
# Pause before continuing
echo "Press Enter to continue to the given examples..."
read

# Run the given error examples
./stock examples/errors/error1 1
echo ""
# Pause before continuing
echo "Press Enter to continue to the given examples..."
read
./stock examples/errors/error2 1
echo ""
# Pause before continuing
echo "Press Enter to continue to the given examples..."
read
./stock examples/errors/error3 1
echo ""
# Pause before continuing
echo "Press Enter to continue to the given examples..."
read

# Run the checker on the examples that should be correct
./stock -checker examples/build examples/build.log
echo ""
# Pause before continuing
echo "Press Enter to continue to the given examples..."
read
./stock -checker examples/seller examples/seller.log
echo ""
# Pause before continuing
echo "Press Enter to continue to the given examples..."
read

# Run the checker on the example that should be incorrect
./stock -checker examples/checkererror/testchecker examples/checkererror/testchecker.log
