#!/usr/bin/env bash
# Test script with pipelines
echo "line1" > /tmp/aush_test_pipeline.txt
echo "line2" >> /tmp/aush_test_pipeline.txt
echo "line3" >> /tmp/aush_test_pipeline.txt
cat /tmp/aush_test_pipeline.txt | grep line2
rm -f /tmp/aush_test_pipeline.txt
