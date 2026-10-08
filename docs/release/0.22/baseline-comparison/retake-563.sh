#!/bin/bash
source ~/.cargo/env
export WEBDRIVER_PATH=/usr/bin/true BROWSER_PATH=/usr/bin/true
/home/joaquin/osl-bench/timing-563-retake.sh /home/joaquin/osl-bench/b563/wt-0213 /home/joaquin/osl-bench/b563/timing-0213-retake.log /home/joaquin/osl-bench/profiles-0213-563.txt
/home/joaquin/osl-bench/timing-563-retake.sh /home/joaquin/osl-bench/b563/wt-main /home/joaquin/osl-bench/b563/timing-main-retake.log /home/joaquin/osl-bench/profiles-022-retake-563.txt
