@echo off



call "cargo" "run" "--" "-t" "test_script_1.yaml" "-r" "-b=1.0" "-o=0.0"
