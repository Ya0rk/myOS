#!/bin/sh

echo "#### OS COMP TEST GROUP START ltp-glibc ####"

# Define target directory
target_dir="/glibc/ltp/testcases/bin"

backip="
access01
accept01 accept4_01
add_key01 add_key04
adjtimex01
alarm02 alarm03 alarm05 alarm06 alarm07

bpf_map01

capget01
chown01 chown03 
clock_getres01 
clock_gettime02
clock_nanosleep04
clone01 clone03 clone06 clone07 clone08
close01
creat01
confstr01

dup01
dup02
dup03
dup04
dup07
dup202
dup203
dup204
dup207
dup3_01
dup3_02

epoll_create01 epoll_create1_01 epoll_ctl03
exit02

faccessat01 faccessat02
fallocate03
fanotify08
fchdir02
fchmod04
fchown01 fchown02 fchown03 fchown04 fchown05
fchownat01 fchownat02
fcntl02 fcntl02_64 fcntl03 fcntl03_64 fcntl04 fcntl04_64 fcntl05 fcntl05_64 fcntl08 fcntl08_64
fcntl12 fcntl12_64 fcntl29 fcntl29_64 fcntl34 fcntl34_64 fcntl36 fcntl36_64
flistxattr03
flock01 flock03 flock04 flock06 fork10
fork01 fork03 fork04 fork07 fork08 fpathconf01
fstat02 fstat02_64 fstat03 fstat03_64
fstatfs02 fstatfs02_64
ftruncate01 ftruncate01_64
futex_wait01
futex_wake01
futex_cmp_requeue02

getdomainname01
geteuid01 geteuid02
gethostname01
getitimer01 getitimer02
getpagesize01
getpeername01
getpgid01
getpgrp01
getpid02
getppid02
getrandom02 getrandom03 getrandom04 
getrlimit01 getrlimit02
getrusage01 getrusage02
gettid02
gettimeofday01 gettimeofday02
getuid01 getuid03

in6_01
inotify_init1_01 inotify_init1_02
ioprio_set02
io_uring01

kcmp01 kcmp03
keyctl04

lftest
llseek02 llseek03
lseek01
lseek07

madvise01 madvise05 madvise10
memset01
mesgq_nstest
mincore02 mincore03
mkdir05
mkdirat01
mlock01 mlock04
mmap02 mmap06 mmap09 mmap19
mtest01
munlock01
mknod01
mknod02
mq_open01

open01 open02 open10 open03 open11
openat01 openat202

pipe01 pipe06 pipe10 pipe14 pipe2_01 pipe11
prctl01
pselect03 pselect03_64
pread01
pread01_64
process_vm_readv03
process_vm_writev02
pwrite02 pwrite02_64   pwrite04 pwrite04_64
pwritev202 pwritev202_64

read01 read02 read04
readahead01
readdir01
readlink01 readlink03
readlinkat01 readlinkat02
readv01 readv02
recvmsg02
request_key01
request_key05
rmdir01
rt_sigsuspend01

sbrk01 sbrk02
semctl07
semget01
setdomainname01 setdomainname02
setfsuid02
setgid01
setgroups01 setgroups02
sethostname01 sethostname02
setregid01 setregid04
setreuid01
sendfile03 sendfile03_64
sendfile06 sendfile06_64 sendfile08 sendfile08_64
setrlimit02 setrlimit03
setsockopt01 setsockopt03
settimeofday02
setxattr02
shmctl02 shmctl03 shmctl04 shmctl07 shmctl08
shmem_2nstest
shmnstest
setuid01 signal01 signal02 signal03 signal04 signal05
sigaltstack02
sigwait01
socket02 socket01
socketpair01
splice01 splice03
stat01 stat01_64 stat02 stat02_64
statx01 statx02
symlink02
syscall01

time01
times01
tkill01

uname01
unlinkat01
utsname01 utsname04

wait01 wait02 wait401
waitpid01 waitpid04
write01 write02
waitid01 waitid04 waitid05 waitid06 waitid11
"

# List of test cases (no array, using space-separated string)
# you can add prog you want to test.if it succeed, you can put it in var backip.
# from jdlu
ltp_cases="
accept01
"


for case in $backip; do
  file="$target_dir/$case"

  if [ -f "$file" ]; then
    echo "RUN LTP CASE $case"

    "$file"
    ret=$?

    echo "FAIL LTP CASE $case : $ret"
    /musl/busybox rm -f "$file"
  fi
done

echo "#### OS COMP TEST GROUP END ltp-glibc ####"