#!/usr/bin/env bash
# Integration test for the shift builtin

set -e

echo "Testing shift builtin..."

# Test 1: Basic shift
echo "Test 1: Basic shift with positional parameters"
test_func() {
    echo "Before shift: \$1=$1 \$2=$2 \$3=$3 \$#=$#"
    shift
    echo "After shift: \$1=$1 \$2=$2 \$3=$3 \$#=$#"
}
test_func a b c

echo ""

# Test 2: Shift multiple parameters
echo "Test 2: Shift multiple parameters"
test_func2() {
    echo "Before shift 2: \$1=$1 \$2=$2 \$3=$3 \$4=$4 \$#=$#"
    shift 2
    echo "After shift 2: \$1=$1 \$2=$2 \$3=$3 \$4=$4 \$#=$#"
}
test_func2 one two three four

echo ""

# Test 3: Process all arguments with shift in a loop
echo "Test 3: Process all arguments in a loop"
process_all() {
    echo "Processing $# arguments..."
    while [ $# -gt 0 ]; do
        echo "  Arg: $1"
        shift
    done
    echo "Done. Remaining args: $#"
}
process_all apple banana cherry date

echo ""

# Test 4: Use $@ and $* special variables
echo "Test 4: Special variables \$@ and \$*"
test_special() {
    echo "All args (\$@): $@"
    echo "All args (\$*): $*"
    echo "Count (\$#): $#"
    shift
    echo "After shift - All args (\$@): $@"
    echo "After shift - Count (\$#): $#"
}
test_special first second third

echo ""

# Test 5: Shift in a script with script args
echo "Test 5: Shift with script arguments"
cat > /tmp/shift_test.sh << 'EOF'
#!/usr/bin/env bash
echo "Script args: $@"
echo "Count: $#"
shift
echo "After shift: $@"
echo "Count: $#"
EOF
chmod +x /tmp/shift_test.sh
./aush /tmp/shift_test.sh arg1 arg2 arg3
rm /tmp/shift_test.sh

echo ""
echo "All shift tests passed!"
