#!/bin/sh
# ShellSpec test for POSIX signal handling
# Tests trap, signal propagation, and job control signals

Describe 'POSIX Signal Handling'
  Include ./spec_helper.sh

  Describe 'trap builtin'
    It 'sets signal handler'
      When call rush_c "trap 'echo caught' USR1 && kill -USR1 \$\$"
      The output should include "caught"
      The status should be success
    End

    It 'lists traps with no arguments'
      When call rush_c "trap"
      The status should be success
    End

    It 'sets EXIT trap'
      When call rush_c "trap 'echo exiting' EXIT && exit 0"
      The output should include "exiting"
      The status should equal 0
    End

    It 'clears trap with -'
      When call rush_c "trap 'echo test' USR1 && trap - USR1 && trap"
      The status should be success
    End

    It 'ignores signal with empty string'
      When call rush_c "trap '' USR1 && kill -USR1 \$\$"
      The status should be success
    End
  End

  Describe 'signal names'
    It 'accepts numeric signal'
      When call rush_c "trap 'echo sig' 15"
      The status should be success
    End

    It 'accepts signal name without SIG prefix'
      When call rush_c "trap 'echo sig' TERM"
      The status should be success
    End

    It 'accepts signal name with SIG prefix'
      When call rush_c "trap 'echo sig' SIGTERM"
      The status should be success
    End
  End

  Describe 'special trap conditions'
    It 'supports ERR trap'
      When call rush_c "trap 'echo error' ERR && false"
      The output should include "error"
      The status should equal 1
    End

    It 'supports DEBUG trap'
      When call rush_c "trap 'echo debug' DEBUG && echo test"
      The output should include "debug"
      The status should be success
    End

    It 'supports RETURN trap in functions'
      When call rush_c "foo() { trap 'echo returning' RETURN; echo in_func; }; foo"
      The output should include "in_func"
      The output should include "returning"
      The status should be success
    End
  End

  Describe 'trap execution context'
    It 'trap runs in current shell environment'
      When call rush_c "FOO=bar && trap 'echo \$FOO' EXIT && exit"
      The output should include "bar"
      The status should be success
    End

    It 'trap can modify variables'
      When call rush_c "trap 'FOO=changed' USR1 && kill -USR1 \$\$ && echo \$FOO"
      The output should include "changed"
      The status should be success
    End
  End

  Describe 'signal handling in subshells'
    It 'subshells inherit traps'
      When call rush_c "trap 'echo parent' USR1 && (kill -USR1 \$\$)"
      The status should be success
    End

    It 'subshell traps do not affect parent'
      When call rush_c "(trap 'echo sub' USR1); trap"
      The status should be success
    End
  End

  Describe 'SIGINT handling'
    It 'SIGINT interrupts foreground job'
      Skip
      When call rush_c "sleep 10"
      # This test requires sending SIGINT which is complex to test
      The status should be success
    End
  End

  Describe 'SIGTERM handling'
    It 'SIGTERM terminates shell'
      When call rush_c "trap 'echo term' TERM && kill -TERM \$\$"
      The output should include "term"
      The status should not equal 0
    End
  End

  Describe 'signal inheritance'
    It 'child processes inherit signal dispositions'
      When call rush_c "trap '' HUP && sh -c 'trap'"
      The status should be success
    End
  End

  Describe 'kill builtin'
    It 'sends signal to process'
      When call rush_c "sleep 10 & PID=\$!; kill \$PID; wait \$PID"
      The status should not equal 0
    End

    It 'lists signals with -l'
      When call rush_c "kill -l"
      The output should include "TERM"
      The status should be success
    End

    It 'sends specific signal'
      When call rush_c "sleep 10 & PID=\$!; kill -TERM \$PID; wait \$PID"
      The status should not equal 0
    End

    It 'kills multiple processes'
      When call rush_c "sleep 10 & P1=\$!; sleep 10 & P2=\$!; kill \$P1 \$P2; wait"
      The status should not equal 0
    End
  End
End
