macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

macro_rules! count_tts {
    ($($tts:tt)*) => {0usize $(+ replace_expr!($tts 1usize))*};
}

macro_rules! builtins {
    ($(($name:literal, $func:path, $($args:literal),*)),+) => {
		lazy_static! {
			static ref ARRAY: [(&'static str, Vec<&'static str>); (count_tts!{$($name)+})] = {
				[$(($name, vec![$($args,)*])),+]
			};
		}
		pub fn lookup(name: &str, args: &[Arg]) -> core::result::Result<(String, $crate::ops::FnOp<f64>), $crate::Error> {
			match name {
				$($name => $func(args, arguments($name)),)+
				_ => Err($crate::Error::UndefinedVariable(name.to_owned())),
			}
		}
	};
}

pub(super) use builtins;
