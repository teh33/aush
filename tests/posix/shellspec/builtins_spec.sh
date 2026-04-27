#!/bin/sh
# ShellSpec test for POSIX builtin commands
# Tests all required POSIX shell builtins

Describe 'POSIX Builtin Commands'
  Include ./spec_helper.sh

  # cd builtin
  Describe 'cd'
    It 'changes directory'
      When call rush_c "cd /tmp && pwd"
      The output should equal "/tmp"
      The status should be success
    End

    It 'supports cd -'
      When call rush_c "cd /tmp && cd /usr && cd - > /dev/null && pwd"
      The output should equal "/tmp"
      The status should be success
    End

    It 'uses HOME when no argument'
      When call rush_c "HOME=/tmp && cd && pwd"
      The output should equal "/tmp"
      The status should be success
    End
  End

  # pwd builtin
  Describe 'pwd'
    It 'prints current directory'
      When call rush_c "cd /tmp && pwd"
      The output should equal "/tmp"
      The status should be success
    End

    It 'supports -L flag (logical)'
      When call rush_c "pwd -L"
      The status should be success
    End

    It 'supports -P flag (physical)'
      When call rush_c "pwd -P"
      The status should be success
    End
  End

  # echo builtin
  Describe 'echo'
    It 'prints arguments'
      When call rush_c "echo hello world"
      The output should equal "hello world"
      The status should be success
    End

    It 'handles no arguments'
      When call rush_c "echo"
      The output should equal ""
      The status should be success
    End

    It 'handles special characters'
      When call rush_c "echo 'hello\nworld'"
      The status should be success
    End
  End

  # exit builtin
  Describe 'exit'
    It 'exits with code 0'
      When call rush_c "exit 0"
      The status should equal 0
    End

    It 'exits with specified code'
      When call rush_c "exit 42"
      The status should equal 42
    End

    It 'exits with last command exit code when no argument'
      When call rush_c "false; exit"
      The status should equal 1
    End
  End

  # export builtin
  Describe 'export'
    It 'exports variables'
      When call rush_c "export FOO=bar && sh -c 'echo \$FOO'"
      The output should equal "bar"
      The status should be success
    End

    It 'exports without value'
      When call rush_c "FOO=bar && export FOO && sh -c 'echo \$FOO'"
      The output should equal "bar"
      The status should be success
    End

    It 'lists exported variables with no arguments'
      When call rush_c "export"
      The status should be success
    End
  End

  # readonly builtin
  Describe 'readonly'
    It 'makes variables readonly'
      When call rush_c "readonly FOO=bar && FOO=baz"
      The status should be failure
    End

    It 'lists readonly variables'
      When call rush_c "readonly"
      The status should be success
    End
  End

  # unset builtin
  Describe 'unset'
    It 'unsets variables'
      When call rush_c "FOO=bar && unset FOO && echo \$FOO"
      The output should equal ""
      The status should be success
    End

    It 'unsets functions'
      When call rush_c "foo() { echo bar; } && unset -f foo && type foo"
      The status should be failure
    End
  End

  # set builtin
  Describe 'set'
    It 'sets positional parameters'
      When call rush_c "set -- a b c && echo \$1 \$2 \$3"
      The output should equal "a b c"
      The status should be success
    End

    It 'supports -e option (errexit)'
      When call rush_c "set -e && false && echo should_not_print"
      The status should be failure
    End

    It 'supports -u option (nounset)'
      When call rush_c "set -u && echo \$UNDEFINED_VAR"
      The status should be failure
    End

    It 'supports -x option (xtrace)'
      When call rush_c "set -x && echo test" 2>&1
      The status should be success
    End

    It 'supports +o to turn off options'
      When call rush_c "set -e && set +e && false && echo ok"
      The output should equal "ok"
      The status should be success
    End
  End

  # shift builtin
  Describe 'shift'
    It 'shifts positional parameters'
      When call rush_c "set -- a b c && shift && echo \$1"
      The output should equal "b"
      The status should be success
    End

    It 'shifts by n positions'
      When call rush_c "set -- a b c d && shift 2 && echo \$1"
      The output should equal "c"
      The status should be success
    End

    It 'fails when shifting too many'
      When call rush_c "set -- a && shift 2"
      The status should be failure
    End
  End

  # eval builtin
  Describe 'eval'
    It 'evaluates arguments as command'
      When call rush_c "eval 'echo hello'"
      The output should equal "hello"
      The status should be success
    End

    It 'handles variable expansion'
      When call rush_c "CMD='echo test' && eval \$CMD"
      The output should equal "test"
      The status should be success
    End
  End

  # exec builtin
  Describe 'exec'
    It 'replaces shell with command'
      When call rush_c "exec echo test"
      The output should equal "test"
      The status should be success
    End
  End

  # return builtin
  Describe 'return'
    It 'returns from function with code'
      When call rush_c "foo() { return 42; } && foo"
      The status should equal 42
    End

    It 'returns with 0 if no argument'
      When call rush_c "foo() { return; } && foo"
      The status should equal 0
    End
  End

  # read builtin
  Describe 'read'
    It 'reads input into variable'
      When call sh -c "echo 'test input' | aush -c 'read VAR && echo \$VAR'"
      The output should equal "test input"
      The status should be success
    End

    It 'reads multiple variables'
      When call sh -c "echo 'a b c' | aush -c 'read X Y Z && echo \$X \$Y \$Z'"
      The output should equal "a b c"
      The status should be success
    End

    It 'assigns remainder to last variable'
      When call sh -c "echo 'one two three four' | aush -c 'read A B && echo \"\$A|\$B\"'"
      The output should equal "one|two three four"
      The status should be success
    End

    It 'uses REPLY when no variable given'
      When call sh -c "echo 'default' | aush -c 'read && echo \$REPLY'"
      The output should equal "default"
      The status should be success
    End

    It 'uses -r flag for raw mode (preserves backslashes)'
      When call sh -c "printf 'hello\\\\nworld\\n' | aush -c 'read -r VAR && echo \"\$VAR\"'"
      The output should equal 'hello\nworld'
      The status should be success
    End

    It 'processes backslashes without -r flag'
      When call sh -c "printf 'hello\\\\nworld\\n' | aush -c 'read VAR && echo \"\$VAR\"'"
      The output should equal "hellonworld"
      The status should be success
    End

    It 'returns 1 on EOF'
      When call sh -c "printf '' | aush -c 'read VAR; echo \$?'"
      The output should equal "1"
    End

    # Note: Custom IFS is tested in unit tests (test_builtin_read_with_stdin_custom_ifs)
    # Pipeline-based tests are tricky since read runs in subshell
  End

  # true and false builtins
  Describe 'true and false'
    It 'true returns 0'
      When call rush_c "true"
      The status should equal 0
    End

    It 'false returns 1'
      When call rush_c "false"
      The status should equal 1
    End
  End

  # colon builtin
  Describe ': (colon)'
    It 'does nothing and returns 0'
      When call rush_c ":"
      The status should equal 0
    End

    It 'expands arguments but does nothing'
      When call rush_c ": \$((1+1))"
      The status should equal 0
    End
  End

  # test/[ builtin
  Describe 'test and ['
    It 'tests string equality'
      When call rush_c "test 'a' = 'a'"
      The status should equal 0
    End

    It 'tests string inequality'
      When call rush_c "test 'a' != 'b'"
      The status should equal 0
    End

    It 'tests numeric equality'
      When call rush_c "test 5 -eq 5"
      The status should equal 0
    End

    It 'tests numeric inequality'
      When call rush_c "test 5 -ne 4"
      The status should equal 0
    End

    It 'tests file existence'
      When call rush_c "test -e /tmp"
      The status should equal 0
    End

    It 'tests directory existence'
      When call rush_c "test -d /tmp"
      The status should equal 0
    End

    It 'works with [ ] syntax'
      When call rush_c "[ 1 -eq 1 ]"
      The status should equal 0
    End
  End

  # trap builtin
  Describe 'trap'
    It 'sets signal handler'
      When call rush_c "trap 'echo caught' INT && kill -INT \$\$"
      The status should be success
    End

    It 'lists traps'
      When call rush_c "trap"
      The status should be success
    End

    It 'clears trap with -'
      When call rush_c "trap 'echo test' INT && trap - INT"
      The status should be success
    End
  End

  # wait builtin
  Describe 'wait'
    It 'waits for background job'
      When call rush_c "sleep 0.1 & wait"
      The status should equal 0
    End

    It 'waits for specific job'
      When call rush_c "sleep 0.1 & PID=\$! && wait \$PID"
      The status should equal 0
    End
  End

  # break and continue builtins
  Describe 'break and continue'
    It 'break exits loop'
      When call rush_c "for i in 1 2 3; do echo \$i; break; done"
      The output should equal "1"
      The status should equal 0
    End

    It 'continue skips iteration'
      When call rush_c "for i in 1 2 3; do [ \$i = 2 ] && continue; echo \$i; done"
      The output should include "1"
      The output should include "3"
      The output should not include "2"
      The status should equal 0
    End
  End

  # hash builtin (command caching)
  Describe 'hash'
    It 'manages command hash table'
      When call rush_c "hash"
      The status should be success
    End
  End

  # umask builtin
  Describe 'umask'
    It 'displays current umask'
      When call rush_c "umask"
      The status should be success
    End

    It 'sets umask'
      When call rush_c "umask 022 && umask"
      The output should equal "0022"
      The status should be success
    End
  End

  # command builtin
  Describe 'command'
    It 'runs command without function lookup'
      When call rush_c "echo() { true; } && command echo test"
      The output should equal "test"
      The status should be success
    End

    It 'supports -v option'
      When call rush_c "command -v echo"
      The status should be success
    End
  End

  # type builtin
  Describe 'type'
    It 'displays command type'
      When call rush_c "type echo"
      The status should be success
    End

    It 'identifies builtins'
      When call rush_c "type cd"
      The output should include "builtin"
      The status should be success
    End

    It 'identifies functions'
      When call rush_c "foo() { :; } && type foo"
      The output should include "function"
      The status should be success
    End
  End
End
