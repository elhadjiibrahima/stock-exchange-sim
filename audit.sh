#!/usr/bin/env zsh

# Build the binary
go build -o stock

# Run own examples
./stock examples/zen
./stock examples/macguffin
./stock examples/matryushka 0.002
echo ""

# Run the given examples
./stock examples/simple 1
./stock examples/build 10
./stock examples/seller 10
./stock examples/fertilizer 0.001
./stock examples/fertilizer 0.0003
echo ""

# Run the given error examples
./stock examples/errors/error1 1
echo ""
./stock examples/errors/error2 1
echo ""
./stock examples/errors/error3 1
echo ""

# Run the checker on the examples that should be correct
./stock -checker examples/build examples/build.log
echo ""
./stock -checker examples/seller examples/seller.log
echo ""

# Run the checker on the example that should be incorrect
./stock -checker examples/checkererror/testchecker examples/checkererror/testchecker.log
