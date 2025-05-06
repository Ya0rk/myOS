

// use paste::paste;

/// impl a is_XX func for flag XX, returning a bool value
#[macro_export]
macro_rules! impl_flag_checker {
    ($($vis:vis $flag:ident),+) => {
        paste::paste!{
            $(  
                #[allow(unused)]  
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
                $vis fn [<set_ $flag>](&mut self, val: bool) -> &mut Self {
                    self.set(Self::$flag, val);
                    self
                }
            )+    
        }
    };
}