#!/bin/sh
# ShellSpec test for POSIX shell functions
# Tests function definition, calling, scope, and return

Describe 'POSIX Shell Functions'
  Include ./spec_helper.sh

  Describe 'function definition'
    It 'defines function with name() syntax'
      When call rush_c "foo() { echo bar; }; foo"
      The output should equal "bar"
      The status should be success
    End

    It 'allows empty function body'
      When call rush_c "foo() { :; }; foo"
      The status should be success
    End

    It 'can define multiple functions'
      When call rush_c "a() { echo a; }; b() { echo b; }; a; b"
      The output should include "a"
      The output should include "b"
      The status should be success
    End
  End

  Describe 'function calling'
    It 'calls function by name'
      When call rush_c "greet() { echo hello; }; greet"
      The output should equal "hello"
      The status should be success
    End

    It 'passes arguments to function'
      When call rush_c "greet() { echo hello \$1; }; greet world"
      The output should equal "hello world"
      The status should be success
    End

    It 'accesses all positional parameters'
      When call rush_c "show() { echo \$1 \$2 \$3; }; show a b c"
      The output should equal "a b c"
      The status should be success
    End

    It 'uses $# for argument count'
      When call rush_c "count() { echo \$#; }; count a b c"
      The output should equal "3"
      The status should be success
    End

    It 'uses $* for all arguments'
      When call rush_c "all() { echo \$*; }; all a b c"
      The output should equal "a b c"
      The status should be success
    End

    It 'uses $@ for all arguments separately'
      When call rush_c "each() { for x in \"\$@\"; do echo \$x; done; }; each a b c"
      The output should include "a"
      The output should include "b"
      The output should include "c"
      The status should be success
    End
  End

  Describe 'function return values'
    It 'returns 0 by default'
      When call rush_c "foo() { echo test; }; foo"
      The status should equal 0
    End

    It 'returns last command exit status'
      When call rush_c "foo() { false; }; foo"
      The status should equal 1
    End

    It 'returns explicit value with return'
      When call rush_c "foo() { return 42; }; foo"
      The status should equal 42
    End

    It 'return exits function immediately'
      When call rush_c "foo() { return 0; echo should_not_print; }; foo"
      The output should equal ""
      The status should equal 0
    End
  End

  Describe 'function variable scope'
    It 'functions access global variables'
      When call rush_c "FOO=bar; show() { echo \$FOO; }; show"
      The output should equal "bar"
      The status should be success
    End

    It 'functions can modify global variables'
      When call rush_c "FOO=init; change() { FOO=changed; }; change; echo \$FOO"
      The output should equal "changed"
      The status should be success
    End

    It 'positional parameters are function-local'
      When call rush_c "set -- global; foo() { echo \$1; }; foo local"
      The output should equal "local"
      The status should be success
    End

    It 'shift affects function parameters'
      When call rush_c "foo() { shift; echo \$1; }; foo a b c"
      The output should equal "b"
      The status should be success
    End

    It 'set affects function parameters'
      When call rush_c "foo() { set -- x y z; echo \$1; }; foo a b c"
      The output should equal "x"
      The status should be success
    End
  End

  Describe 'recursive functions'
    It 'calls itself recursively'
      When call rush_c "fact() { [ \$1 -le 1 ] && echo 1 || echo \$((\$1 * \$(fact \$((\$1 - 1))))); }; fact 5"
      The output should equal "120"
      The status should be success
    End

    It 'handles deep recursion'
      When call rush_c "count() { [ \$1 -eq 0 ] && echo done || { echo \$1; count \$((\$1-1)); }; }; count 10"
      The output should include "10"
      The output should include "done"
      The status should be success
    End
  End

  Describe 'function redefinition'
    It 'allows redefining functions'
      When call rush_c "foo() { echo old; }; foo() { echo new; }; foo"
      The output should equal "new"
      The status should be success
    End

    It 'last definition wins'
      When call rush_c "foo() { echo 1; }; foo() { echo 2; }; foo() { echo 3; }; foo"
      The output should equal "3"
      The status should be success
    End
  End

  Describe 'unset -f'
    It 'removes function definition'
      When call rush_c "foo() { echo test; }; unset -f foo; foo"
      The status should not equal 0
    End

    It 'does not affect variables'
      When call rush_c "FOO=bar; foo() { :; }; unset -f foo; echo \$FOO"
      The output should equal "bar"
      The status should be success
    End
  End

  Describe 'type command with functions'
    It 'identifies function'
      When call rush_c "foo() { :; }; type foo"
      The output should include "function"
      The status should be success
    End

    It 'shows function definition'
      When call rush_c "foo() { echo test; }; type foo"
      The status should be success
    End
  End

  Describe 'command -v with functions'
    It 'reports function name'
      When call rush_c "foo() { :; }; command -v foo"
      The output should equal "foo"
      The status should be success
    End
  End

  Describe 'function and command same name'
    It 'function takes precedence over command'
      When call rush_c "echo() { printf 'custom\n'; }; echo test"
      The output should equal "custom"
      The status should be success
    End

    It 'command builtin bypasses function'
      When call rush_c "echo() { printf 'custom\n'; }; command echo test"
      The output should equal "test"
      The status should be success
    End
  End

  Describe 'nested function calls'
    It 'calls functions from within functions'
      When call rush_c "a() { b; }; b() { echo nested; }; a"
      The output should equal "nested"
      The status should be success
    End

    It 'maintains call stack'
      When call rush_c "a() { echo a; b; }; b() { echo b; c; }; c() { echo c; }; a"
      The output should include "a"
      The output should include "b"
      The output should include "c"
      The status should be success
    End
  End

  Describe 'function with pipelines'
    It 'function output can be piped'
      When call rush_c "gen() { echo test; }; gen | cat"
      The output should equal "test"
      The status should be success
    End

    It 'function can receive piped input'
      When call rush_c "proc() { cat; }; echo test | proc"
      The output should equal "test"
      The status should be success
    End
  End

  Describe 'function with redirections'
    It 'redirects function output'
      When call rush_c "gen() { echo test; }; gen > /tmp/aush_test_$$; cat /tmp/aush_test_$$; rm -f /tmp/aush_test_$$"
      The output should equal "test"
      The status should be success
    End

    It 'redirects function input'
      When call rush_c "echo test > /tmp/aush_test_$$; proc() { cat; }; proc < /tmp/aush_test_$$; rm -f /tmp/aush_test_$$"
      The output should equal "test"
      The status should be success
    End
  End

  Describe 'function local variables'
    Skip 'local keyword creates function-scoped variables'
      When call rush_c "foo() { local X=local; echo \$X; }; X=global; foo; echo \$X"
      The output should include "local"
      The output should include "global"
      The status should be success
    End
  End

  Describe 'export in functions'
    It 'function can export variables'
      When call rush_c "foo() { export BAR=baz; }; foo; sh -c 'echo \$BAR'"
      The output should equal "baz"
      The status should be success
    End
  End
End
