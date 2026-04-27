#!/bin/sh
# ShellSpec test for POSIX I/O redirection
# Tests all forms of redirection

Describe 'POSIX I/O Redirection'
  Include ./spec_helper.sh

  Describe 'output redirection'
    It 'redirects stdout with >'
      When call rush_c "echo test > /tmp/aush_test_$$; cat /tmp/aush_test_$$; rm -f /tmp/aush_test_$$"
      The output should equal "test"
      The status should be success
    End

    It 'truncates existing file with >'
      When call rush_c "echo old > /tmp/aush_test_$$; echo new > /tmp/aush_test_$$; cat /tmp/aush_test_$$; rm -f /tmp/aush_test_$$"
      The output should equal "new"
      The status should be success
    End

    It 'appends with >>'
      When call rush_c "echo a > /tmp/aush_test_$$; echo b >> /tmp/aush_test_$$; cat /tmp/aush_test_$$; rm -f /tmp/aush_test_$$"
      The output should include "a"
      The output should include "b"
      The status should be success
    End

    It 'redirects stderr with 2>'
      When call rush_c "sh -c 'echo error >&2' 2> /tmp/aush_test_$$; cat /tmp/aush_test_$$; rm -f /tmp/aush_test_$$"
      The output should equal "error"
      The status should be success
    End

    It 'appends stderr with 2>>'
      When call rush_c "sh -c 'echo e1 >&2' 2> /tmp/aush_test_$$; sh -c 'echo e2 >&2' 2>> /tmp/aush_test_$$; cat /tmp/aush_test_$$; rm -f /tmp/aush_test_$$"
      The output should include "e1"
      The output should include "e2"
      The status should be success
    End

    It 'redirects both stdout and stderr with &>'
      When call rush_c "{ echo out; sh -c 'echo err >&2'; } &> /tmp/aush_test_$$; cat /tmp/aush_test_$$; rm -f /tmp/aush_test_$$"
      The output should include "out"
      The output should include "err"
      The status should be success
    End

    It 'redirects stdout to stderr with >&2'
      When call rush_c "echo test >&2" 2>&1
      The output should equal "test"
      The status should be success
    End

    It 'redirects stderr to stdout with 2>&1'
      When call rush_c "sh -c 'echo error >&2' 2>&1"
      The output should equal "error"
      The status should be success
    End
  End

  Describe 'input redirection'
    It 'redirects stdin with <'
      When call rush_c "echo content > /tmp/aush_test_$$; cat < /tmp/aush_test_$$; rm -f /tmp/aush_test_$$"
      The output should equal "content"
      The status should be success
    End

    It 'reads from file'
      When call rush_c "echo test > /tmp/aush_test_$$; read VAR < /tmp/aush_test_$$; echo \$VAR; rm -f /tmp/aush_test_$$"
      The output should equal "test"
      The status should be success
    End
  End

  Describe 'here-documents'
    It 'supports basic here-document'
      When call rush_c "cat <<EOF
hello
world
EOF"
      The output should include "hello"
      The output should include "world"
      The status should be success
    End

    It 'expands variables in here-document'
      When call rush_c "FOO=bar && cat <<EOF
value: \$FOO
EOF"
      The output should include "value: bar"
      The status should be success
    End

    It 'does not expand when delimiter is quoted'
      When call rush_c "FOO=bar && cat <<'EOF'
value: \$FOO
EOF"
      The output should include "value: \$FOO"
      The status should be success
    End

    It 'supports here-string with <<<'
      When call rush_c "cat <<< 'hello'"
      The output should equal "hello"
      The status should be success
    End

    It 'strips leading tabs with <<-'
      When call rush_c "cat <<-EOF
\thello
EOF"
      The output should equal "hello"
      The status should be success
    End
  End

  Describe 'file descriptor manipulation'
    It 'duplicates file descriptors with <&'
      When call rush_c "exec 3<&0"
      The status should be success
    End

    It 'duplicates file descriptors with >&'
      When call rush_c "exec 3>&1"
      The status should be success
    End

    It 'closes file descriptors with <&-'
      When call rush_c "exec 3>&1 && exec 3>&-"
      The status should be success
    End

    It 'closes file descriptors with >&-'
      When call rush_c "exec 3>&1 && exec 3>&-"
      The status should be success
    End
  End

  Describe 'redirection ordering'
    It 'processes redirections left to right'
      When call rush_c "echo test > /tmp/aush_test_$$ 2>&1 1>/dev/null; cat /tmp/aush_test_$$; rm -f /tmp/aush_test_$$"
      The status should be success
    End

    It 'applies redirections before command execution'
      When call rush_c "echo test > /tmp/aush_test_$$; cat /tmp/aush_test_$$; rm -f /tmp/aush_test_$$"
      The output should equal "test"
      The status should be success
    End
  End

  Describe 'redirection with builtins'
    It 'redirects builtin output'
      When call rush_c "echo test > /tmp/aush_test_$$; cat /tmp/aush_test_$$; rm -f /tmp/aush_test_$$"
      The output should equal "test"
      The status should be success
    End

    It 'redirects builtin input'
      When call rush_c "echo test > /tmp/aush_test_$$; read VAR < /tmp/aush_test_$$; echo \$VAR; rm -f /tmp/aush_test_$$"
      The output should equal "test"
      The status should be success
    End
  End

  Describe 'redirection with pipelines'
    It 'redirects pipeline output'
      When call rush_c "echo test | cat > /tmp/aush_test_$$; cat /tmp/aush_test_$$; rm -f /tmp/aush_test_$$"
      The output should equal "test"
      The status should be success
    End

    It 'redirects pipeline input'
      When call rush_c "echo test > /tmp/aush_test_$$; cat < /tmp/aush_test_$$ | cat; rm -f /tmp/aush_test_$$"
      The output should equal "test"
      The status should be success
    End
  End

  Describe 'noclobber option'
    It 'prevents overwriting files with >| when noclobber is set'
      When call rush_c "set -C && echo old > /tmp/aush_test_$$ && echo new > /tmp/aush_test_$$"
      The status should be failure
    End

    It 'allows overwriting with >| when noclobber is set'
      When call rush_c "set -C && echo old > /tmp/aush_test_$$ && echo new >| /tmp/aush_test_$$; cat /tmp/aush_test_$$; rm -f /tmp/aush_test_$$"
      The output should equal "new"
      The status should be success
    End
  End

  Describe '/dev/null'
    It 'discards output to /dev/null'
      When call rush_c "echo test > /dev/null"
      The output should equal ""
      The status should be success
    End

    It 'reads empty from /dev/null'
      When call rush_c "cat < /dev/null"
      The output should equal ""
      The status should be success
    End
  End
End
