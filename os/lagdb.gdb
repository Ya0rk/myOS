file ./target/loongarch64-unknown-none/release/os
target remote localhost:1234

b rust_main
c
echo __trap_from_user\n
b *0x9000000000202000  
echo trap.S:209 csrwr $t1, CSR_ERA\n
b *0x9000000000202278  
display/2gi $pc
display/x $t1
display/x $ra