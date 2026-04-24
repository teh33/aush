#!/bin/sh
# Shell Capability Benchmark
# Tests: fork/exec, pipes, I/O, builtins, arithmetic, strings, subshells, memory

set -e
ITERATIONS=${ITERATIONS:-1000}
TMPDIR=${TMPDIR:-/tmp}
TMPFILE="$TMPDIR/shellbench_$$"
RESULTS="$TMPDIR/shellbench_results_$$"

cleanup() { rm -f "$TMPFILE"* 2>/dev/null; }
trap cleanup EXIT

log() {
    printf "%-40s %s\n" "$1" "$2"
}

benchmark() {
    name=$1
    shift
    start=$(date +%s%N 2>/dev/null || date +%s)
    "$@"
    end=$(date +%s%N 2>/dev/null || date +%s)
    
    if [ ${#start} -gt 10 ]; then
        # nanoseconds available
        elapsed=$(( (end - start) / 1000000 ))
    else
        # seconds only
        elapsed=$(( (end - start) * 1000 ))
    fi
    log "$name" "${elapsed}ms"
    echo "$name: ${elapsed}ms" >> "$RESULTS"
}

echo "=== SHELL BENCHMARK ==="
echo "Shell: $0"
echo "PID: $$"
echo "Iterations: $ITERATIONS"
echo ""

# 1. FORK/EXEC STRESS
echo "--- Process Creation ---"
benchmark "Fork/exec (/bin/true)" sh -c '
    i=0
    while [ $i -lt '$ITERATIONS' ]; do
        /bin/true
        i=$((i + 1))
    done
'

benchmark "Fork/exec (echo)" sh -c '
    i=0
    while [ $i -lt '$ITERATIONS' ]; do
        echo "test" >/dev/null
        i=$((i + 1))
    done
'

# 2. BUILTIN VS EXTERNAL
echo ""
echo "--- Builtin vs External ---"
benchmark "Builtin echo" sh -c '
    i=0
    while [ $i -lt '$ITERATIONS' ]; do
        echo "test string here" >/dev/null
        i=$((i + 1))
    done
'

benchmark "External /bin/echo" sh -c '
    i=0
    while [ $i -lt '$ITERATIONS' ]; do
        /bin/echo "test string here" >/dev/null
        i=$((i + 1))
    done
'

# 3. ARITHMETIC
echo ""
echo "--- Arithmetic ---"
benchmark "Integer arithmetic" sh -c '
    i=0; x=0
    while [ $i -lt '$ITERATIONS' ]; do
        x=$(( (i * 7 + 3) / 2 % 100 ))
        i=$((i + 1))
    done
'

benchmark "Nested arithmetic" sh -c '
    i=0
    while [ $i -lt 1000 ]; do
        x=$(( ((i + 1) * (i - 1) + (i / 2)) % 1000 ))
        i=$((i + 1))
    done
'

# 4. STRING MANIPULATION
echo ""
echo "--- String Operations ---"
benchmark "String assignment/concat" sh -c '
    i=0
    while [ $i -lt '$ITERATIONS' ]; do
        a="hello"
        b="world"
        c="$a $b $i"
        i=$((i + 1))
    done
'

benchmark "Variable expansion" sh -c '
    foo="abcdefghijklmnopqrstuvwxyz"
    i=0
    while [ $i -lt '$ITERATIONS' ]; do
        x="${foo}${foo}${foo}"
        i=$((i + 1))
    done
'

# 5. I/O REDIRECTION
echo ""
echo "--- I/O Redirection ---"
benchmark "Write temp file" sh -c '
    i=0
    while [ $i -lt 1000 ]; do
        echo "line $i" > "$TMPFILE.$i"
        i=$((i + 1))
    done
    rm -f "$TMPFILE".*
'

benchmark "Append to file" sh -c '
    i=0
    while [ $i -lt 10000 ]; do
        echo "x" >> "$TMPFILE"
        i=$((i + 1))
    done
'

benchmark "Read from file" sh -c '
    for i in $(seq 1 1000); do echo $i; done > "$TMPFILE"
    while read line; do
        :
    done < "$TMPFILE"
'

benchmark "Here-document" sh -c '
    i=0
    while [ $i -lt 1000 ]; do
        cat <<EOF >/dev/null
This is a here document
with multiple lines
and variable $i
EOF
        i=$((i + 1))
    done
'

# 6. PIPES
echo ""
echo "--- Pipelines ---"
benchmark "Simple pipe (echo | cat)" sh -c '
    i=0
    while [ $i -lt '$ITERATIONS' ]; do
        echo "test" | cat >/dev/null
        i=$((i + 1))
    done
'

benchmark "Multi-stage pipeline" sh -c '
    i=0
    while [ $i -lt 1000 ]; do
        echo "hello world" | tr a-z A-Z | sed "s/HELLO/HI/" | cat >/dev/null
        i=$((i + 1))
    done
'

benchmark "Pipe with while read" sh -c '
    seq 1 1000 | while read n; do
        :
    done
'

# 7. SUBSHELLS & COMMAND SUBSTITUTION
echo ""
echo "--- Subshells ---"
benchmark "Command substitution" sh -c '
    i=0
    while [ $i -lt 1000 ]; do
        x=$(echo $i)
        i=$((i + 1))
    done
'

benchmark "Backtick substitution" sh -c '
    i=0
    while [ $i -lt 1000 ]; do
        x=`echo $i`
        i=$((i + 1))
    done
'

benchmark "Subshell (parens)" sh -c '
    i=0
    while [ $i -lt 1000 ]; do
        (exit 0)
        i=$((i + 1))
    done
'

# 8. CONTROL FLOW
echo ""
echo "--- Control Flow ---"
benchmark "If/then/else" sh -c '
    i=0
    while [ $i -lt '$ITERATIONS' ]; do
        if [ $((i % 2)) -eq 0 ]; then
            x=even
        else
            x=odd
        fi
        i=$((i + 1))
    done
'

benchmark "Case statement" sh -c '
    i=0
    while [ $i -lt 1000 ]; do
        case $((i % 5)) in
            0) x=zero ;;
            1) x=one ;;
            2) x=two ;;
            3) x=three ;;
            4) x=four ;;
        esac
        i=$((i + 1))
    done
'

# 9. FUNCTIONS (if supported)
echo ""
echo "--- Functions ---"
benchmark "Function calls" sh -c '
    dummy() { return 0; }
    i=0
    while [ $i -lt '$ITERATIONS' ]; do
        dummy
        i=$((i + 1))
    done
'

benchmark "Recursive function (depth 100)" sh -c '
    recurse() {
        if [ "$1" -le 0 ]; then return; fi
        recurse $(( $1 - 1 ))
    }
    i=0
    while [ $i -lt 100 ]; do
        recurse 100
        i=$((i + 1))
    done
'

# 10. ENVIRONMENT / MEMORY
echo ""
echo "--- Environment ---"
benchmark "Export variables" sh -c '
    i=0
    while [ $i -lt 1000 ]; do
        export VAR_$i="value_$i"
        i=$((i + 1))
    done
    unset $(env | grep "^VAR_" | cut -d= -f1)
'

benchmark "Large environment export" sh -c '
    big=$(printf "%01000d" 0)
    export BIGVAR="$big"
    unset BIGVAR
'

# 11. GLOBBING
echo ""
echo "--- Pathname Expansion ---"
benchmark "Glob expansion" sh -c '
    mkdir -p "$TMPDIR/globtest_$$"
    cd "$TMPDIR/globtest_$$"
    for i in $(seq 1 500); do touch "file_$i.txt"; done
    for i in $(seq 1 50); do
        for f in *.txt; do : ; done >/dev/null 2>&1
    done
    cd "$TMPDIR"
    rm -rf "$TMPDIR/globtest_$$"
'

# 12. BACKGROUND JOBS
echo ""
echo "--- Job Control ---"
benchmark "Background jobs" sh -c '
    i=0
    while [ $i -lt 100 ]; do
        /bin/true &
        i=$((i + 1))
    done
    wait
'

# 13. EDGE CASES
echo ""
echo "--- Edge Cases ---"
benchmark "Empty command" sh -c '
    i=0
    while [ $i -lt 1000 ]; do
        : 
        i=$((i + 1))
    done
'

benchmark "Many arguments" sh -c '
    /bin/true a b c d e f g h i j k l m n o p q r s t u v w x y z \
              a b c d e f g h i j k l m n o p q r s t u v w x y z \
              a b c d e f g h i j k l m n o p q r s t u v w x y z
' 1000

benchmark "Deeply nested quotes" sh -c '
    i=0
    while [ $i -lt 1000 ]; do
        eval "x=\"hello world\"" 2>/dev/null || x="hello world"
        i=$((i + 1))
    done
'

# Summary
echo ""
echo "=== RESULTS SUMMARY ==="
if [ -f "$RESULTS" ]; then
    cat "$RESULTS"
    rm -f "$RESULTS"
fi
echo ""
echo "Benchmark complete."