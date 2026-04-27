#!/bin/sh
# ShellSpec test for POSIX pipelines
# Tests pipeline execution, exit codes, and job control

Describe 'POSIX Pipelines'
  Include ./spec_helper.sh

  Describe 'basic pipelines'
    It 'pipes stdout from one command to stdin of another'
      When call rush_c "echo test | cat"
      The output should equal "test"
      The status should be success
    End

    It 'chains multiple commands'
      When call rush_c "echo hello | cat | cat | cat"
      The output should equal "hello"
      The status should be success
    End

    It 'processes data through pipeline'
      When call rush_c "echo 'a\nb\nc' | sort"
      The status should be success
    End

    It 'filters output'
      When call rush_c "echo -e 'foo\nbar\nbaz' | grep bar"
      The output should equal "bar"
      The status should be success
    End
  End

  Describe 'pipeline exit status'
    It 'returns exit status of last command'
      When call rush_c "true | false"
      The status should equal 1
    End

    It 'returns 0 if last command succeeds'
      When call rush_c "false | true"
      The status should equal 0
    End

    It 'propagates failure'
      When call rush_c "echo test | grep nomatch"
      The status should equal 1
    End
  End

  Describe 'pipefail option'
    It 'returns failure if any command fails with set -o pipefail'
      When call rush_c "set -o pipefail && false | true"
      The status should equal 1
    End

    It 'returns success if all commands succeed'
      When call rush_c "set -o pipefail && true | true"
      The status should equal 0
    End
  End

  Describe 'pipeline with builtins'
    It 'pipes from builtin'
      When call rush_c "echo test | cat"
      The output should equal "test"
      The status should be success
    End

    It 'pipes to builtin'
      When call rush_c "echo test | read VAR; echo \$VAR"
      The status should be success
    End
  End

  Describe 'pipeline stderr handling'
    It 'stderr not piped by default'
      When call rush_c "sh -c 'echo error >&2' | cat" 2>&1
      The output should equal "error"
      The status should be success
    End

    It 'can redirect stderr to stdout before pipe'
      When call rush_c "sh -c 'echo error >&2' 2>&1 | cat"
      The output should equal "error"
      The status should be success
    End
  End

  Describe 'complex pipelines'
    It 'handles long pipelines'
      When call rush_c "echo 1 | cat | cat | cat | cat | cat | cat"
      The output should equal "1"
      The status should be success
    End

    It 'combines with redirections'
      When call rush_c "echo test | cat > /tmp/aush_test_$$; cat /tmp/aush_test_$$; rm -f /tmp/aush_test_$$"
      The output should equal "test"
      The status should be success
    End

    It 'works in subshells'
      When call rush_c "(echo test | cat)"
      The output should equal "test"
      The status should be success
    End

    It 'works in command groups'
      When call rush_c "{ echo test | cat; }"
      The output should equal "test"
      The status should be success
    End
  End

  Describe 'background jobs'
    It 'runs job in background with &'
      When call rush_c "sleep 0.1 &"
      The status should be success
    End

    It 'returns immediately from background job'
      When call rush_c "sleep 10 & echo immediate"
      The output should equal "immediate"
      The status should be success
    End

    It 'sets $! to background job PID'
      When call rush_c "sleep 0.1 & echo \$! | grep -E '^[0-9]+$'"
      The status should be success
    End
  End

  Describe 'job control with wait'
    It 'waits for background job to complete'
      When call rush_c "sleep 0.1 & wait"
      The status should be success
    End

    It 'waits for specific job'
      When call rush_c "sleep 0.1 & PID=\$!; wait \$PID"
      The status should be success
    End

    It 'returns exit status of waited job'
      When call rush_c "sh -c 'exit 42' & wait"
      The status should equal 42
    End
  End

  Describe 'jobs builtin'
    It 'lists background jobs'
      When call rush_c "sleep 1 & jobs"
      The status should be success
    End

    It 'shows job status'
      When call rush_c "sleep 1 & jobs -l"
      The status should be success
    End
  End

  Describe 'fg and bg builtins'
    Skip 'fg brings job to foreground'
      When call rush_c "sleep 0.1 & fg"
      The status should be success
    End

    Skip 'bg continues stopped job in background'
      When call rush_c "sleep 1 & jobs"
      The status should be success
    End
  End

  Describe 'pipeline with loops'
    It 'pipes into while loop'
      When call rush_c "echo -e '1\n2\n3' | while read line; do echo \$line; done"
      The output should include "1"
      The output should include "2"
      The output should include "3"
      The status should be success
    End

    It 'pipes from for loop'
      When call rush_c "for i in 1 2 3; do echo \$i; done | cat"
      The output should include "1"
      The output should include "2"
      The output should include "3"
      The status should be success
    End
  End

  Describe 'command execution in pipelines'
    It 'each command runs in separate process'
      When call rush_c "FOO=bar | echo \$FOO"
      The output should equal ""
      The status should be success
    End

    It 'environment changes do not persist after pipeline'
      When call rush_c "FOO=init; echo | FOO=changed; echo \$FOO"
      The output should equal "init"
      The status should be success
    End
  End

  Describe 'pipeline buffering'
    It 'handles large data through pipeline'
      When call rush_c "seq 1 1000 | wc -l"
      The output should equal "1000"
      The status should be success
    End
  End

  Describe 'named pipes (FIFOs)'
    Skip 'can use named pipes'
      When call rush_c "mkfifo /tmp/aush_fifo_$$; echo test > /tmp/aush_fifo_$$ & cat < /tmp/aush_fifo_$$; rm /tmp/aush_fifo_$$"
      The output should equal "test"
      The status should be success
    End
  End
End
