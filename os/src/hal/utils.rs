

use paste::paste;

/// impl a is_XX func for flag XX, returning a bool value
#[macro_export]
macro_rules! impl_flag_checker {
    ($($vis:vis $flag:ident),+) => {
        paste::paste!{
            $(  
                #[allow(unused)] 
                // #[inline(always)] 
                $vis fn [<is_ $flag>](&self) -> bool {
                    self.contains(Self::$flag)
                }
            )+
        }
        
    };
}

/// impl a set_XX func for flag XX, returning a &mut Self
#[macro_export]
macro_rules! impl_flag_setter {
    ($($vis:vis $flag:ident),+) => {
        paste::paste!{  
            $(
                #[allow(unused)]
                // #[inline(always)] 
                $vis fn [<set_ $flag>](&mut self, val: bool) -> &mut Self {
                    self.set(Self::$flag, val);
                    self
                }
            )+    
        }
    };
}


// macro_rules! generate_GPRs_64 {
//     { $($tt:tt);+  $(;)?} => {
//         __generate_GPRs_64_inner!{0, $($tt;)+}
        
//     };
// }
// macro_rules! __generate_GPRs_64_inner {
//     { $ordinal:expr, ($name:ident, $doc:expr); $($tt:tt);* $(;)?} => {
//         __generate_GP_register_64_no_dup!($name, $doc, $ordinal);
//         __generate_GPRs_64_inner!{$ordinal + 1, $($tt;)*}
//     };
//     { $ordinal:expr, ($name:ident dup $dup:expr, $doc:expr); $($tt:tt);* $(;)?} => {
//         __generate_GP_register_64_with_dup!($name, $doc, $ordinal, $dup);
//         __generate_GPRs_64_inner!{$ordinal + $dup, $($tt;)*}
//     };
//     { $ordinal:expr, } => {

//     };
// }

// macro_rules! __generate_GPR_64_step {
//     ( $ordinal:expr, ($name:ident, $doc:expr)) => {
        
//     };
// }

// #[cfg(target_arch="loongarch64")]
// macro_rules! __format_GPR_ordinal_and_alias {
//     ($ordinal:expr, $alias:ident) => {
//         format!("General-Purpose Register r{} a.k.a {}\n", $ordinal, stringify!($alias))
//     };
// }

// macro_rules! __generate_GP_register_64_no_dup {
//     ($name:ident, $doc:expr, $ordinal:expr) => {
//         #[doc = __format_GPR_ordinal_and_alias!($ordinal, $name)]
//         #[doc = $doc]
//         let $name: usize;
//     };
// }

// macro_rules! __generate_GP_register_64_with_dup {
//     ($name:ident, $doc:expr, $ordinal:expr, 0) => {
//         __generate_GP_register_64_no_dup!($name, $doc, $ordinal)
//     };
//     ($name:ident, $doc:expr, $ordinal:expr, $dup:expr) => {
//         paste::paste!{
//             __generate_GP_register_64_with_dup!($name, $doc, $ordinal, $dup - 1)
//             __generate_GP_register_64_no_dup!([<$name, $dup>], $doc, $ordinal + $dup)            
//         }
//     }
// }

// struct GP{
    
// }
// generate_GPRs_64!{
//     (zero, "zero bits");
//     (reg, "annotation");
//     (s dup 9, "static registers");
// }

