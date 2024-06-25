//! Convenience Macros
//!
//! This module contains macros for use by command authors.

/// Returns an `Ok` `MoltResult`.
///
/// If called with no arguments, returns an empty value as the `Ok` result.
/// If called with one argument, returns the argument as the `Ok` result, converting it
/// to a value automatically.
/// If called with two or more arguments, computes the `Ok` result using
/// `format!()`; the first argument is naturally the format string.
///
/// # Examples
///
/// ```
/// use molt::*;
///
/// // Return the empty result
/// fn func1() -> MoltResult {
///     // ...
///     molt_ok!()
/// }
///
/// assert_eq!(func1(), Ok(Value::empty()));
///
/// // Return an arbitrary value
/// fn func2() -> MoltResult {
///     // ...
///     molt_ok!(17)
/// }
///
/// assert_eq!(func2(), Ok(17.into()));
///
/// // Return a formatted value
/// fn func3() -> MoltResult {
///     // ...
///     molt_ok!("The answer is {}", 17)
/// }
///
/// assert_eq!(func3(), Ok("The answer is 17".into()));
/// ```
#[macro_export]
macro_rules! molt_ok {
    () => (
        Ok($crate::Value::empty())
    );
    ($arg:expr) => (
        Ok($crate::Value::from($arg))
    );
    ($($arg:tt)*) => (
        Ok($crate::Value::from(format!($($arg)*)))
    )
}

/// Returns an `Error` `MoltResult`.  The error message is formatted
/// as with `format!()`.
///
/// If called with one argument, the single argument is used as the error message.
/// If called with more than one argument, the first is a `format!()` format string,
/// and the remainder are the values to format.
///
/// This macro wraps the [`Exception::molt_err`](types/struct.Exception.html#method.molt_err)
/// method.
///
/// # Examples
///
/// ```
/// use molt::*;
///
/// // Return a simple error message
/// fn err1() -> MoltResult {
///     // ...
///     molt_err!("error message")
/// }
///
/// let result = err1();
/// assert!(result.is_err());
///
/// let exception = result.err().unwrap();
/// assert!(exception.is_error());
/// assert_eq!(exception.value(), "error message".into());
///
/// // Return a formatted error
/// fn err2() -> MoltResult {
///    // ...
///    molt_err!("invalid value: {}", 17)
/// }
///
/// let result = err2();
/// assert!(result.is_err());
///
/// let exception = result.err().unwrap();
/// assert!(exception.is_error());
/// assert_eq!(exception.value(), "invalid value: 17".into());
/// ```
#[macro_export]
macro_rules! molt_err {
    ($arg:expr) => (
        Err($crate::Exception::molt_err($crate::Value::from($arg)))
    );
    ($($arg:tt)*) => (
        Err($crate::Exception::molt_err($crate::Value::from(format!($($arg)*))))
    )
}

#[macro_export]
macro_rules! molt_err_help {
    ($arg:expr) => {{
      let mut e = $crate::Exception::molt_err($crate::Value::from($arg));
      e.to_help();
      Err(e)
    }};
    ($($arg:tt)*) => {{
      let mut e = $crate::Exception::molt_err($crate::Value::from(format!($($arg)*)));
      e.to_help();
      Err(e)
    }}
}

/// Returns an `Error` `MoltResult` with a specific error code.  The error message is formatted
/// as with `format!()`.
///
/// The macro requires two or more arguments.  The first argument is always the error code.
/// If called with two arguments, the second is the error message.
/// If called with more than two arguments, the second is a `format!()` format string and
/// the remainder are the values to format.
///
/// This macro wraps
/// the [`Exception::molt_err2`](types/struct.Exception.html#method.molt_err2)
/// method.
///
/// # Examples
///
/// ```
/// use molt::*;
///
/// // Throw a simple error
/// fn throw1() -> MoltResult {
///     // ...
///     molt_throw!("MYCODE", "error message")
/// }
///
/// let result = throw1();
/// assert!(result.is_err());
///
/// let exception = result.err().unwrap();
/// assert!(exception.is_error());
/// assert_eq!(exception.value(), "error message".into());
/// assert_eq!(exception.error_code(), "MYCODE".into());
///
/// // Return a formatted error
/// fn throw2() -> MoltResult {
///    // ...
///    molt_throw!("MYCODE", "invalid value: {}", 17)
/// }
///
/// let result = throw2();
/// assert!(result.is_err());
///
/// let exception = result.err().unwrap();
/// assert!(exception.is_error());
/// assert_eq!(exception.value(), "invalid value: 17".into());
/// assert_eq!(exception.error_code(), "MYCODE".into());
/// ```
#[macro_export]
macro_rules! molt_throw {
    ($code:expr, $msg:expr) => (
        Err($crate::Exception::molt_err2($crate::Value::from($code), $crate::Value::from($msg)))
    );
    ($code:expr, $($arg:tt)*) => (
        Err($crate::Exception::molt_err2($crate::Value::from($code), $crate::Value::from(format!($($arg)*))))
    )
}

#[macro_export]
macro_rules! join_strings {
  () => {
      ""
  };
  ($a:expr $(,)?) => {
      $a
  };
  ($a:expr, $b:expr $(,)?) => {
      concat!($a, " or ", $b)
  };
  ($a:expr, $($rest:expr),+ $(,)?) => {
      concat!($a, ", ", join_strings!($($rest),+))
  };
}

/// A Molt command that has subcommands is called an _ensemble_ command.  In Rust code,
/// the ensemble is defined as an array of `Subcommand` structs, each one mapping from
/// a subcommand name to the implementing [`CommandFunc`].  For more information,
/// see the discussion of command definition in [The Molt Book] and the [`interp`] module.
///
/// The tuple fields are the subcommand's name and implementing [`CommandFunc`].
///
/// [The Molt Book]: https://wduquette.github.io/molt/
/// [`interp`]: ../interp/index.html
/// [`CommandFunc`]: type.CommandFunc.html
///
/// Calls a subcommand of the current command, looking up its name in an array of
/// `Subcommand` tuples.
///
/// The subcommand, if found, is called with the same `context_ids` and `argv` as its
/// parent ensemble.  `subc` is the index of the subcommand's name in the `argv` array;
/// in most cases it will be `1`, but it is possible to define subcommands with
/// subcommands of their own.  The `subcommands` argument is a borrow of an array of
/// `Subcommand` records, each defining a subcommand's name and `CommandFunc`.
///
/// If the subcommand name is found in the array, the matching `CommandFunc` is called.
/// otherwise, the error message gives the ensemble syntax.  If an invalid subcommand
/// name was provided, the error message includes the valid options.
///
/// See the implementation of the `array` command in `commands.rs` and the
/// [module level documentation](index.html) for examples.
#[macro_export]
macro_rules! _gen_subcommand_generic {
  ($subc:expr, [ $( ($cmd_name:tt, $cmd_func:expr$(,)?) ),* $(,)?] $(,)?) => {
    {
      #[inline]
      fn f<Ctx:'static>(interp: &mut Interp<Ctx>, argv: &[Value]) -> MoltResult {
        check_args($subc, argv, $subc + 1, 0, "subcommand ?arg ...?")?;
        let sub_name = argv[$subc].as_str();
        match sub_name {
          $(
            $cmd_name => $cmd_func(interp, argv),
          )*
          _ => molt_err!("unknown or ambiguous subcommand \"{}\", must be:\n{}.", sub_name, join_strings!( $($cmd_name,)* )),
        }
      }
      f
    }
  }
}

/// A Molt command that has subcommands is called an _ensemble_ command.  In Rust code,
/// the ensemble is defined as an array of `Subcommand` structs, each one mapping from
/// a subcommand name to the implementing [`CommandFunc`].  For more information,
/// see the discussion of command definition in [The Molt Book] and the [`interp`] module.
///
/// The tuple fields are the subcommand's name and implementing [`CommandFunc`].
///
/// [The Molt Book]: https://wduquette.github.io/molt/
/// [`interp`]: ../interp/index.html
/// [`CommandFunc`]: type.CommandFunc.html
///
/// Calls a subcommand of the current command, looking up its name in an array of
/// `Subcommand` tuples.
///
/// The subcommand, if found, is called with the same `context_ids` and `argv` as its
/// parent ensemble.  `subc` is the index of the subcommand's name in the `argv` array;
/// in most cases it will be `1`, but it is possible to define subcommands with
/// subcommands of their own.  The `subcommands` argument is a borrow of an array of
/// `Subcommand` records, each defining a subcommand's name and `CommandFunc`.
///
/// If the subcommand name is found in the array, the matching `CommandFunc` is called.
/// otherwise, the error message gives the ensemble syntax.  If an invalid subcommand
/// name was provided, the error message includes the valid options.
///
/// See the implementation of the `array` command in `commands.rs` and the
/// [module level documentation](index.html) for examples.
#[macro_export]
macro_rules! gen_subcommand {
  ($ctx_type:ty, $subc:expr, [ $( ($cmd_name:tt, $cmd_space:tt, $cmd_func:expr, $cmd_help:expr$(,)?) ),* $(,)?] $(,)?) => {
    {
      #[inline]
      fn f(interp: &mut Interp<$ctx_type>, argv: &[Value]) -> MoltResult {
        check_args($subc, argv, $subc + 1, 0, "subcommand ?arg ...?")?;
        let sub_name = argv[$subc].as_str();
        const HELP_MSG: &str = join_helps_subcmd!( $( [$cmd_name,$cmd_space,$cmd_help], )* );
        match sub_name {
          $(
            $cmd_name => $cmd_func(interp, argv),
          )*
          "-help" => molt_ok!("usage:\n{}",HELP_MSG),
          _ => molt_err_help!("unknown subcommand in \"{} {}\", usage:\n{}", argv[0..$subc].iter().map(|v|v.as_str()).collect::<Vec<&str>>().join(" "),sub_name,HELP_MSG ),
        }
      }
      f
    }
  }
}

#[macro_export]
macro_rules! join_helps_subcmd {
  (  ) => {
      ""
  };
  // Base case: single element, no trailing newline
  ( [$first:expr, $second:expr, $third:expr]$(,)? ) => {
      concat!("  ", $first, "  ", $second, $third, "\n  -help")
  };
  // Recursive case: multiple elements
  ( [$first:expr, $second:expr, $third:expr], $( [$rest_first:expr, $rest_second:expr, $rest_third:expr] ),+$(,)? ) => {
      concat!(
        "  ", $first, "  ", $second, $third, "\n",
        join_helps_subcmd!($( [$rest_first, $rest_second, $rest_third] ),+)
      )
  };
}

#[macro_export]
macro_rules! join_helps {
  (  ) => {
      ""
  };
  // Base case: single element, no trailing newline
  ( [$first:expr, $second:expr, $third:expr]$(,)? ) => {
      concat!("  ", $first, "  ", $second, $third, "\n  help  [-all]")
  };
  // Recursive case: multiple elements
  ( [$first:expr, $second:expr, $third:expr], $( [$rest_first:expr, $rest_second:expr, $rest_third:expr] ),+$(,)? ) => {
      concat!(
        "  ", $first, "  ", $second, $third, "\n",
          join_helps!($( [$rest_first, $rest_second, $rest_third] ),+)
      )
  };
}

#[macro_export]
macro_rules! gen_command {
  ($ctx_type:ty, [ $( ($native_name:tt, $native_func:expr $(,)?) ),* $(,)?], [ $( ($embedded_name:tt, $embedded_space:tt, $embedded_func:expr, $embedded_help:tt $(,)?) ),* $(,)?] $(,)?) => {
    Command::new(
      {fn f(name: &str, interp: &mut Interp<$ctx_type>, argv: &[Value]) -> MoltResult {
        const HELP_MSG: &str = join_helps!( $( [$embedded_name,$embedded_space,$embedded_help], )* );
        match name {
          // NOTICE: Default native commands
          _APPEND => cmd_append(interp, argv),
          _ARRAY => cmd_array(interp, argv),
          _ASSERT_EQ => cmd_assert_eq(interp, argv),
          _BREAK => cmd_break(interp, argv),
          _CATCH => cmd_catch(interp, argv),
          _CONTINUE => cmd_continue(interp, argv),
          _DICT => cmd_dict(interp, argv),
          _ERROR => cmd_error(interp, argv),
          _EXPR => cmd_expr(interp, argv),
          _FOR => cmd_for(interp, argv),
          _FOREACH => cmd_foreach(interp, argv),
          _GLOBAL => cmd_global(interp, argv),
          _IF => cmd_if(interp, argv),
          _INCR => cmd_incr(interp, argv),
          _INFO => cmd_info(interp, argv),
          _JOIN => cmd_join(interp, argv),
          _LAPPEND => cmd_lappend(interp, argv),
          _LINDEX => cmd_lindex(interp, argv),
          _LIST => cmd_list(interp, argv),
          _LLENGTH => cmd_llength(interp, argv),
          _PROC => cmd_proc(interp, argv),
          _PUTS => cmd_puts(interp, argv),
          _RENAME => cmd_rename(interp, argv),
          _RETURN => cmd_return(interp, argv),
          _SET => cmd_set(interp, argv),
          _STRING => cmd_string(interp, argv),
          _THROW => cmd_throw(interp, argv),
          _TIME => cmd_time(interp, argv),
          _UNSET => cmd_unset(interp, argv),
          _WHILE => cmd_while(interp, argv),
          "help" => {
            if let Some(v)= argv.get(1){
              if v.as_str()=="-all"{
                let proc_command_names = interp.proc_command_names();
                if proc_command_names.is_empty(){
                  return molt_ok!("usage of {}:\ntcl:\n  {}\n{}:\n{}", interp.name,interp.native_command_names(),interp.name,HELP_MSG);
                }else{
                  return molt_ok!("usage of {}:\ntcl:\n  {}\n{}:\n{}\nprocedure:\n  {}", interp.name,interp.native_command_names(),interp.name,HELP_MSG,proc_command_names);
                }
              }
            }
            molt_ok!("usage of {}:\n{}",interp.name,HELP_MSG)},
          // NOTICE: Extra native commands
          $(
            $native_name => $native_func(interp, argv),
          )*
          // NOTICE: Embedded commands
          $(
            $embedded_name => $embedded_func(interp, argv),
          )*
          // NOTICE: Proc commands
          other => {
            if let Some(proc) = interp.get_proc(other) {
              proc.clone().execute(interp, argv)
            } else {
              let proc_command_names = interp.proc_command_names();
              if proc_command_names.is_empty(){
                molt_err_help!("unknown command \"{}\", valid commands:\ntcl:\n  {}\n{}:\n{}", name,interp.native_command_names(),interp.name,HELP_MSG)
              }else{
                molt_err_help!("unknown command \"{}\", valid commands:\ntcl:\n  {}\n{}:\n{}\nprocedure:\n  {}", name,interp.native_command_names(),interp.name,HELP_MSG,proc_command_names)
              }
            }
          }
        }
      }
      f as fn(&str, &mut Interp<$ctx_type>, &[Value]) -> MoltResult
      },
      {fn f(name: &str, interp: &Interp<$ctx_type>) -> Option<CommandType> {
        match name {
          _APPEND => Some(CommandType::Native),
          _ARRAY => Some(CommandType::Native),
          _ASSERT_EQ => Some(CommandType::Native),
          _BREAK => Some(CommandType::Native),
          _CATCH => Some(CommandType::Native),
          _CONTINUE => Some(CommandType::Native),
          _DICT => Some(CommandType::Native),
          _ERROR => Some(CommandType::Native),
          _EXPR => Some(CommandType::Native),
          _FOR => Some(CommandType::Native),
          _FOREACH => Some(CommandType::Native),
          _GLOBAL => Some(CommandType::Native),
          _IF => Some(CommandType::Native),
          _INCR => Some(CommandType::Native),
          _INFO => Some(CommandType::Native),
          _JOIN => Some(CommandType::Native),
          _LAPPEND => Some(CommandType::Native),
          _LINDEX => Some(CommandType::Native),
          _LIST => Some(CommandType::Native),
          _LLENGTH => Some(CommandType::Native),
          _PROC => Some(CommandType::Native),
          _PUTS => Some(CommandType::Native),
          _RENAME => Some(CommandType::Native),
          _RETURN => Some(CommandType::Native),
          _SET => Some(CommandType::Native),
          _STRING => Some(CommandType::Native),
          _THROW => Some(CommandType::Native),
          _TIME => Some(CommandType::Native),
          _UNSET => Some(CommandType::Native),
          _WHILE => Some(CommandType::Native),
          $(
            $native_name => Some(CommandType::Native),
          )*
          $(
            $embedded_name => Some(CommandType::Embedded),
          )*
          other => {
            if interp.contains_proc(other) {
              Some(CommandType::Proc)
            } else {
              None
            }
          }
        }
      }
      f as fn(&str, &Interp<$ctx_type>) -> Option<CommandType>
      },
      &[
        _APPEND,
        _ARRAY,
        _ASSERT_EQ,
        _BREAK,
        _CATCH,
        _CONTINUE,
        _DICT,
        _ERROR,
        _EXPR,
        _FOR,
        _FOREACH,
        _GLOBAL,
        _IF,
        _INCR,
        _INFO,
        _JOIN,
        _LAPPEND,
        _LINDEX,
        _LIST,
        _LLENGTH,
        _PROC,
        _PUTS,
        _RENAME,
        _RETURN,
        _SET,
        _STRING,
        _THROW,
        _TIME,
        _UNSET,
        _WHILE,
        $(
            $native_name,
        )*
      ],
      &[
        $(
          $embedded_name,
        )*
      ]
    )
  };
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_molt_ok() {
        let result: MoltResult = molt_ok!();
        assert_eq!(Ok(Value::empty()), result);

        let result: MoltResult = molt_ok!(5);
        assert_eq!(Ok(Value::from(5)), result);

        let result: MoltResult = molt_ok!("Five");
        assert_eq!(Ok(Value::from("Five")), result);

        let result: MoltResult = molt_ok!("The answer is {}.", 5);
        assert_eq!(Ok(Value::from("The answer is 5.")), result);
    }

    #[test]
    fn test_molt_err() {
        check_err(molt_err!("error message"), "error message");
        check_err(molt_err!("error {}", 5), "error 5");
    }

    #[test]
    fn test_molt_throw() {
        check_throw(molt_throw!("MYERR", "error message"), "MYERR", "error message");
        check_throw(molt_throw!("MYERR", "error {}", 5), "MYERR", "error 5");
    }

    fn check_err(result: MoltResult, msg: &str) -> bool {
        match result {
            Err(exception) => exception.is_error() && exception.value() == msg.into(),
            _ => false,
        }
    }

    fn check_throw(result: MoltResult, code: &str, msg: &str) -> bool {
        match result {
            Err(exception) => {
                exception.is_error()
                    && exception.value() == msg.into()
                    && exception.error_code() == code.into()
            }
            _ => false,
        }
    }
}
