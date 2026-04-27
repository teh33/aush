#!/usr/bin/env bash
# Test script with redirections
echo "redirect test" > /tmp/aush_redirect_test.txt
cat /tmp/aush_redirect_test.txt
echo "append test" >> /tmp/aush_redirect_test.txt
cat /tmp/aush_redirect_test.txt
rm -f /tmp/aush_redirect_test.txt
