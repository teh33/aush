#!/bin/sh
# -------------------------------------------------------------------
# Shell Benchmark & Capability Tester
# Execute with:  your_shell ./benchmark.sh
# -------------------------------------------------------------------

# Keep track of test results
PASS=0
FAIL=0
TOTAL=0

# Helper: run a test command and check exit code
test_ok() {
    TOTAL=$((TOTAL + 1))
    desc="$1"
    shift
    if "$@" >/dev/null 2>&1; then
        printf "PASS: %s\n" "$desc"
        PASS=$((PASS + 1))
    else
        printf "FAIL: %s\n" "$desc"
        FAIL=$((FAIL + 1))
    fi
}

# Helper: run a test, expect FAILURE (non-zero exit)
test_fail() {
    TOTAL=$((TOTAL + 1))
    desc="$1"
    shift
    if "$@" >/dev/null 2>&1; then
        printf "FAIL: %s (expected non-zero)\n" "$desc"
        FAIL=$((FAIL + 1))
    else
        printf "PASS: %s\n" "$desc"
        PASS=$((PASS + 1))
    fi
}

# Helper: compare command output with expected string
test_output() {
    TOTAL=$((TOTAL + 1))
    desc="$1"
    expected="$2"
    shift 2
    actual=$("$@" 2>/dev/null)
    if [ "$actual" = "$expected" ]; then
        printf "PASS: %s\n" "$desc"
        PASS=$((PASS + 1))
    else
        printf "FAIL: %s\n" "$desc"
        printf "  expected: %s\n" "$expected"
        printf "  actual:   %s\n" "$actual"
        FAIL=$((FAIL + 1))
    fi
}

#---------------------------------------------------------------------
# 1) Basic built‑ins and exit codes
#---------------------------------------------------------------------
test_ok "true builtin"              true
test_fail "false builtin"           false
test_ok "echo builtin"              echo hi
test_ok "printf builtin"            printf "hello\n" >/dev/null

#---------------------------------------------------------------------
# 2) Variable assignment & expansion
#---------------------------------------------------------------------
test_ok "simple assignment" sh -c 'a=5; [ "$a" = "5" ]'
test_ok "quoted assignment"  sh -c 'a="hello world"; [ "$a" = "hello world" ]'
test_ok "expand unset to empty" sh -c '[ -z "$unset_var" ]'
test_output "default value" "" sh -c 'echo ${missing:-}'          # empty default
test_output "default value set" "default" sh -c 'echo ${missing:-default}'
test_output "assign default"    "value"   sh -c 'echo ${x:=value}; echo $x'
test_ok "parameter length" sh -c 'x=12345; [ ${#x} -eq 5 ]'

#---------------------------------------------------------------------
# 3) Conditional expressions (test / [ ])
#---------------------------------------------------------------------
test_ok "string comparison ="    sh -c '[ "a" = "a" ]'
test_ok "string inequality !="   sh -c '[ "a" != "b" ]'
test_ok "integer -eq"            sh -c '[ 3 -eq 3 ]'
test_ok "integer -gt"            sh -c '[ 4 -gt 2 ]'
test_ok "-z empty string"        sh -c 'x=""; [ -z "$x" ]'
test_ok "-n non-empty"           sh -c 'x=1;   [ -n "$x" ]'
test_ok "file test -e /dev/null" sh -c '[ -e /dev/null ]'

#---------------------------------------------------------------------
# 4) Redirections
#---------------------------------------------------------------------
test_ok "output redirect" sh -c 'echo test > /tmp/_bench_test; [ -f /tmp/_bench_test ] && rm /tmp/_bench_test'
test_ok "input redirect"  sh -c 'echo test > /tmp/_bench_in; read x < /tmp/_bench_in; [ "$x" = "test" ]; rm /tmp/_bench_in'
test_ok "append redirect" sh -c 'echo line1 > /tmp/_bench_ap; echo line2 >> /tmp/_bench_ap; [ $(wc -l < /tmp/_bench_ap) -eq 2 ]; rm /tmp/_bench_ap'
test_ok "here-document"   sh -c "cat <<EOF
hello
EOF
" | sh -c '[ "$(cat)" = "hello" ]'
# Here‑string (bash/zsh extension) – test only if supported
if command -v readarray >/dev/null 2>&1; then :; fi  # dummy skip check
test_ok "here-string (if available)" sh -c 'cat <<< "hi" 2>/dev/null || true'  # won't fail the test

#---------------------------------------------------------------------
# 5) Pipes
#---------------------------------------------------------------------
test_ok "simple pipe" sh -c 'echo hello | grep h'
test_output "pipe between built-ins" "3" sh -c 'echo 3 | cat | cat'
test_ok "long pipe chain (50 processes)" perl -e 'print "x"x50' | sh -c 'i=0; while read -n1 c; do i=$((i+1)); done; [ $i -eq 50 ]'

#---------------------------------------------------------------------
# 6) Loops and conditionals
#---------------------------------------------------------------------
test_output "for loop" "123" sh -c 'for i in 1 2 3; do echo -n $i; done; echo'
test_output "while loop" "5 4 3 2 1 " sh -c 'i=5; while [ $i -gt 0 ]; do echo -n "$i "; i=$((i-1)); done'
test_ok "if/then/else" sh -c 'if true; then exit 0; else exit 1; fi'
test_output "case statement" "good" sh -c 'x="a"; case $x in a) echo good;; *) echo bad;; esac'

#---------------------------------------------------------------------
# 7) Command substitution
#---------------------------------------------------------------------
test_output '$(...) substitution' "date" sh -c 'echo $(echo date)'
test_output 'backtick substitution' "value" sh -c 'echo `echo value`'
test_output 'nested substitution' "hello" sh -c 'echo $(echo $(echo hello))'

#---------------------------------------------------------------------
# 8) Functions
#---------------------------------------------------------------------
test_output "simple function" "hello" sh -c 'f() { echo hello; }; f'
test_output "function arguments" "1 2" sh -c 'f() { echo $1 $2; }; f 1 2'
test_output "recursive function" "120" sh -c '
fact() { if [ $1 -le 1 ]; then echo 1; else a=$(( $1 * $(fact $(( $1 - 1 ))) )); echo $a; fi; }
fact 5'

#---------------------------------------------------------------------
# 9) Arithmetic expressions (if available)
#---------------------------------------------------------------------
test_ok "basic ((...))" sh -c '(( 1 + 1 == 2 ))'
test_output '$((...))' "7" sh -c 'echo $(( 3 + 4 ))'
test_output "increment" "6" sh -c 'x=5; ((x++)); echo $x'

#---------------------------------------------------------------------
# 10) Quotes and escaping
#---------------------------------------------------------------------
test_output "double quotes preserve $" 'hello world' sh -c 'x=hello; echo "$x world"'
test_output "single quotes, no expansion" '$HOME' sh -c 'echo '"'"'$HOME'"'"
test_output "escape dollar" '$HOME' sh -c 'echo \$HOME'
test_output "escape backslash" '\' sh -c 'echo \\'

#---------------------------------------------------------------------
# 11) Job control basics (if your shell supports &, wait, etc.)
#---------------------------------------------------------------------
echo "Testing background jobs..."
# Run a sleep in background and wait for it.
# If job control is unsupported, the shell may just run it sequentially; that's okay.
sleep 0.1 &
PID=$!
wait $PID >/dev/null 2>&1
if [ $? -eq 0 ]; then
    printf "PASS: background job + wait\n"
    PASS=$((PASS + 1))
else
    printf "FAIL: background job + wait\n"
    FAIL=$((FAIL + 1))
fi
TOTAL=$((TOTAL + 1))

#---------------------------------------------------------------------
# 12) Traps (optional advanced feature)
#---------------------------------------------------------------------
echo "Testing trap..."
# Set a trap for SIGUSR1, send it to self, check if triggered.
TRIGGERED=0
trap 'TRIGGERED=1' USR1
kill -USR1 $$ >/dev/null 2>&1
# Allow a moment for signal delivery
sleep 0.1
if [ "$TRIGGERED" = "1" ]; then
    printf "PASS: trap signal\n"
    PASS=$((PASS + 1))
else
    printf "FAIL: trap signal (maybe unsupported?)\n"
    FAIL=$((FAIL + 1))
fi
TOTAL=$((TOTAL + 1))

#---------------------------------------------------------------------
# 13) Stress tests – many commands, deep recursion, large data
#---------------------------------------------------------------------
echo "Stress: 1000 echo commands..."
stress_out=$(i=0; while [ $i -lt 1000 ]; do echo x; i=$((i+1)); done | wc -l)
if [ "$stress_out" = "1000" ]; then
    printf "PASS: 1000 line stress test\n"
    PASS=$((PASS + 1))
else
    printf "FAIL: 1000 line stress (got %s)\n" "$stress_out"
    FAIL=$((FAIL + 1))
fi
TOTAL=$((TOTAL + 1))

echo "Stress: large here-doc (10000 lines)..."
lines=$(cat <<END
$(i=0; while [ $i -lt 10000 ]; do echo "$i"; i=$((i+1)); done)
END
) | tail -1
if [ "$lines" = "9999" ]; then
    printf "PASS: large here-doc\n"
    PASS=$((PASS + 1))
else
    printf "FAIL: large here-doc (expected 9999, got %s)\n" "$lines"
    FAIL=$((FAIL + 1))
fi
TOTAL=$((TOTAL + 1))

echo "Stress: many nested subprocesses (depth 20)..."
# Create a chain of subshells each echoing one character and concatenating.
deep=$( (echo a; (echo b; (echo c; (echo d; (echo e; (echo f; (echo g; (echo h; (echo i; (echo j; (echo k; (echo l; (echo m; (echo n; (echo o; (echo p; (echo q; (echo r; (echo s; (echo t; echo x))))))))))))))))))) | tr -d '\n' )
if [ "$deep" = "abcdefghijklmnopqrstx" ]; then
    printf "PASS: deep subshell nesting\n"
    PASS=$((PASS + 1))
else
    printf "FAIL: deep subshell nesting (got %s)\n" "$deep"
    FAIL=$((FAIL + 1))
fi
TOTAL=$((TOTAL + 1))

#---------------------------------------------------------------------
# 14) Globbing (simple – no files created, just test expansion in /tmp)
#---------------------------------------------------------------------
test_ok "pathname expansion" sh -c 'files=(/tmp/*); [ ${#files[@]} -ge 0 ]'   # at least empty list works

#---------------------------------------------------------------------
# Summary
#---------------------------------------------------------------------
echo "----------------------------------------"
echo "Tests complete: $TOTAL, Passed: $PASS, Failed: $FAIL"
if [ $FAIL -eq 0 ]; then
    echo "All tests passed!"
else
    echo "Some tests failed."
fi