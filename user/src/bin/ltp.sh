#!/bin/sh

echo "#### OS COMP TEST GROUP START ltp-musl ####"

# Define target directory
target_dir="/musl/ltp/testcases/bin"

# List of test cases (no array, using space-separated string)
ltp_cases="readv01 readv02 reboot01 recvmsg02 request_key01 request_key05 rmdir01 sbrk01 sbrk02 sched_get_priority_max01 sched_get_priority_min01 sched_getparam01 sched_getscheduler01 sched_rr_get_interval01 sched_setparam03 semctl07 semget01 gethostname01 getdomainname01 setdomainname01 setdomainname02 sethostname01 sethostname02 setfsuid02 setgid01 setgroups01 setgroups02 abs01 accept01 accept03 accept04 accept4_01 asapi_01 alarm02 alarm03 alarm05 alarm06 alarm07 atof01 bind01 bind02 brk01 capget01 chmod01 chmod03 chmod05 chown01 chown02 chown03 chown05 chroot03 clock_getres01 clock_nanosleep04 close_range02 close01 clone01 clone02 clone03 clone06 clone07 connect01 dup01 dup02 exit01 exit02 fchdir01 fchdir02 fcntl01 fcntl02 getgid01 getcwd01 getcwd02 getpeername01 getpgid01 getpid02 getsockname01 getsockopt01 getitimer01 getitimer02 lseek01 mkdir02 mkdir03 mkdirat01 mkdirat02 nextafter01 open01 open02 openat01 pipe01 rt_sigaction01 sched_setparam01 sched_setparam02 setpgid01 setpgid02 setpgrp01 setregid01 setregid04 setreuid01 setrlimit01 setrlimit02 setrlimit03 setrlimit04 setsid01 setsockopt01 setsockopt03 setsockopt04 settimeofday02 setuid01 setxattr02 sigaction01 sigaction02 sigaltstack01 sigaltstack02 signal01 signal02 signal03 signal04 signal05 sigwait01 socket02 socketpair01 socketpair02 splice01 splice03 splice07 stack_space stat01 stat01_64 stat02 stat02_64 stream01 stream02 stream03 stream04 stream05 string01 syscall01 sysconf01 sysinfo01 sysinfo02 waitpid01 write01 write02"

for case in $ltp_cases; do
  file="$target_dir/$case"

  if [ -f "$file" ]; then
    echo "RUN LTP CASE $case"

    "$file"
    ret=$?

    echo "FAIL LTP CASE $case : $ret"
#   else
#     echo "SKIP LTP CASE $case : not found"
  fi
done

echo "#### OS COMP TEST GROUP END ltp-musl ####"