//! The Value Type
//!
//! The [`Value`] struct is the standard representation of a data value
//! in the Molt language.  It represents a single immutable data value; the
//! data is reference-counted, so instances can be cloned efficiently.  Its
//! content may be any TCL data value: e.g., a number, a list, a string, or a value of
//! an arbitrary type that meets certain requirements.
//!
//! In TCL, "everything is a string": every value is defined by its _string
//! representation_, or _string rep_.  For example, "one two three" is the string rep of a
//! list with three items, the strings "one", "two", and "three".  A string that is a
//! valid string rep for multiple types can be interpreted as any of those types;
//! for example, the string "5" can be used as a string, the integer 5, or a list of one
//! element, the value "5".
//!
//! Internally, the `Value` can also have a `data representation`, or `data rep`, that
//! reflects how the value has been most recently used.  Once a `Value` has been used
//! as a list, it will continue to be efficiently used as a list (until it is used something
//! with a different data rep).
//!
//! # Value is not Sync!
//!
//! A `Value` is associated with a particular `Interp` and changes internally to optimize
//! performance within that `Interp`.  Consequently, `Values` are not `Sync`.  `Values`
//! may be used to pass values between `Interps` in the same thread (at the cost of
//! potential shimmering), but between threads one should pass the value's string rep instead.
//!
//! # Comparisons
//!
//! If two `Value`'s are compared for equality in Rust, Rust compares their string reps;
//! the client may also use the two `Values` as some other type before comparing them.  In
//! TCL expressions the `==` and `!=` operators compare numbers and the
//! `eq` and `ne` operators compare string reps.
//!
//! # Internal Representation
//!
//! "Everything is a string"; thus, every `Value` has a string
//! representation, or _string rep_.  But for efficiency with numbers, lists,
//! and user-defined binary data structures, the `Value` also caches a
//! data representation, or _data rep_.
//!
//! A `Value` can have just a string rep, just a data rep, or both.
//! Like the `Tcl_Obj` in standard TCL, the `Value` is like a two-legged stork: it
//! can stand one leg, the other leg, or both legs.
//!
//! A client can ask the `Value` for its string, which is always available
//! and will be computed from the data rep if it doesn't already exist.  (Once
//! computed, the string rep never changes.)  A client can also ask
//! the `Value` for any other type it desires.  If the requested data rep
//! is already available, it will be returned; otherwise, the `Value` will
//! attempt to parse it from the string_rep, returning an error result on failure.  The
//! most recent data rep is cached for later.
//!
//! For example, consider the following sequence:
//!
//! * A computation yields a `Value` containing the integer 5. The data rep is
//!   a `MoltInt`, and the string rep is undefined.
//!
//! * The client asks for the string, and the string rep "5" is computed.
//!
//! * The client asks for the value's integer value.  It is available and is returned.
//!
//! * The client asks for the value's value as a MoltList.  This is possible, because
//!   the string "5" can be interpreted as a list of one element, the
//!   string "5".  A new data rep is computed and saved, replacing the previous one.
//!
//! With this scheme, long series of computations can be carried
//! out efficiently using only the the data rep, incurring the parsing cost at most
//! once, while preserving TCL's "everything is a string" semantics.
//!
//! **Shimmering**: Converting from one data rep to another is expensive, as it involves parsing
//! the string value.  Performance can suffer if the user's code switches rapidly from one data
//! rep to another, e.g., in a tight loop.  The effect, which is known as "shimmering",
//! can usually be avoided with a little care.  Note that accessing the value's string rep
//! doesn't cause shimmering; the string is always readily available.
//!
//! `Value` handles strings, integers, floating-point values, lists, and a few other things as
//! special cases, since they are part of the language and are so frequently used.
//! In addition, a `Value` can also contain _external types_: Rust types that implement
//! certain traits.
//!
//! # External Types
//!
//! Any type that implements the `std::fmt::Display`, `std::fmt::Debug`,
//! and `std::str::FromStr` traits can be saved in a `Value`.  The struct's
//! `Display` and `FromStr` trait implementations are used to convert between
//! the string rep and data rep.
//!
//! * The `Display` implementation is responsible for producing the value's string rep.
//!
//! * The `FromStr` implementation is responsible for producing the value's data rep from
//!   a string, and so must be able to parse the `Display` implementation's
//!   output.
//!
//! * The string rep should be chosen so as to fit in well with TCL syntax, lest
//!   confusion, quoting hell, and comedy should ensue.  (You'll know it when you
//!   see it.)
//!
//! ## Example
//!
//! For example, the following code shows how to define an external type implementing
//! a simple enum.
//!
//! ```
//! use molt::types::*;
//! use std::fmt;
//! use std::str::FromStr;
//!
//! #[derive(Debug, PartialEq, Copy, Clone)]
//! pub enum Flavor {
//!     SALTY,
//!     SWEET,
//! }
//!
//! impl fmt::Display for Flavor {
//!     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         if *self == Flavor::SALTY {
//!             write!(f, "salty")
//!         } else {
//!             write!(f, "sweet")
//!         }
//!     }
//! }
//!
//! impl FromStr for Flavor {
//!     type Err = String;
//!
//!     fn from_str(value: &str) -> Result<Self, Self::Err> {
//!         let value = value.to_lowercase();
//!
//!         if value == "salty" {
//!             Ok(Flavor::SALTY)
//!         } else if value == "sweet" {
//!             Ok(Flavor::SWEET)
//!         } else {
//!            // The error message doesn't matter to Molt
//!            Err("Not a flavor string".to_string())
//!         }
//!     }
//! }
//!
//! impl Flavor {
//!     /// A convenience: retrieves the enumerated value, converting it from
//!     /// `Option<Flavor>` into `Result<Flavor,Exception>`.
//!     pub fn from_molt(value: &Value) -> Result<Self, Exception> {
//!         if let Some(x) = value.as_copy::<Flavor>() {
//!             Ok(x)
//!         } else {
//!             Err(Exception::molt_err(Value::from("Not a flavor string")))
//!         }
//!     }
//! }
//! ```
//!
//! # Special Implementation Types
//!
//! Values can also be interpreted as two special types, `Script` and `VarName`.  The
//! Interpreter uses the (non-public) `as_script` method to parse script bodies for
//! evaluation; generally this means that a script will get parsed only once.
//!
//! Similarly, `as_var_name` interprets a variable name reference as a `VarName`, which
//! contains the variable name and, optionally, an array index.  This is usually hidden
//! from the extension author by the `var` and `set_var` methods, but it is available if
//! publically if needed.
//!
//! [`Value`]: struct.Value.html

use crate::dict::dict_to_string;
use crate::dict::list_to_dict;
use crate::expr::Datum;
use crate::list::get_list;
use crate::list::list_to_string;
use crate::parser;
use crate::parser::Script;
use crate::types::Exception;
use crate::types::MoltDict;
use crate::types::MoltFloat;
use crate::types::MoltInt;
use crate::types::MoltList;
use crate::types::VarName;
use std::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::hash::Hasher;
use std::rc::Rc;
use std::str::FromStr;

//-----------------------------------------------------------------------------
// Public Data Types

/// The `Value` type. See [the module level documentation](index.html) for more.
#[derive(Clone)]
pub struct Value {
    /// The actual data, to be shared among multiple instances of `Value`.
    inner: Rc<InnerValue>,
}

impl Hash for Value {
    // A Value is hashed according to its string rep; all Values with the same string rep
    // are identical.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

/// The inner value of a `Value`, to be wrapped in an `Rc<T>` so that `Values` can be shared.
#[derive(Debug)]
struct InnerValue {
    string_rep: UnsafeCell<Option<String>>,
    data_rep: RefCell<DataRep>,
}

impl std::fmt::Debug for Value {
    /// The Debug formatter for values.
    ///
    /// TODO: This should indicate something about the data rep as well, especially for
    /// values in which the string rep isn't yet set.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Value[{}]", self.as_str())
    }
}

impl Value {
    /// Creates a value whose `InnerValue` is defined by its string rep.
    fn inner_from_string(str: String) -> Self {
        let inner = InnerValue {
            string_rep: UnsafeCell::new(Some(str)),
            data_rep: RefCell::new(DataRep::None),
        };

        Self {
            inner: Rc::new(inner),
        }
    }

    /// Creates a value whose `InnerValue` is defined by its data rep.
    fn inner_from_data(data: DataRep) -> Self {
        let inner = InnerValue {
            string_rep: UnsafeCell::new(None),
            data_rep: RefCell::new(data),
        };

        Self {
            inner: Rc::new(inner),
        }
    }
}

impl Display for Value {
    /// The `Display` formatter for `Value`.  Outputs the value's string rep.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Eq for Value {}
impl PartialEq for Value {
    /// Two Values are equal if their string representations are equal.  Application code will
    /// often want to compare values numerically.
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<String> for Value {
    /// Creates a new `Value` from the given String.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// let string = String::from("My New String");
    /// let value = Value::from(string);
    /// assert_eq!(value.as_str(), "My New String");
    /// ```
    fn from(str: String) -> Self {
        Value::inner_from_string(str)
    }
}

impl From<&str> for Value {
    /// Creates a new `Value` from the given string slice.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// let value = Value::from("My String Slice");
    /// assert_eq!(value.as_str(), "My String Slice");
    /// ```
    fn from(str: &str) -> Self {
        Value::inner_from_string(str.to_string())
    }
}

impl From<&String> for Value {
    /// Creates a new `Value` from the given string reference.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// let value = Value::from("My String Slice");
    /// assert_eq!(value.as_str(), "My String Slice");
    /// ```
    fn from(str: &String) -> Self {
        Value::inner_from_string(str.to_string())
    }
}

impl From<bool> for Value {
    /// Creates a new `Value` whose data representation is a `bool`.  The value's
    /// string representation will be "1" or "0".
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// let value = Value::from(true);
    /// assert_eq!(value.as_str(), "1");
    ///
    /// let value = Value::from(false);
    /// assert_eq!(value.as_str(), "0");
    /// ```
    fn from(flag: bool) -> Self {
        Value::inner_from_data(DataRep::Bool(flag))
    }
}

impl From<MoltDict> for Value {
    /// Creates a new `Value` whose data representation is a `MoltDict`.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// use molt::types::MoltDict;
    /// use molt::dict::dict_new;
    ///
    /// let mut dict: MoltDict = dict_new();
    /// dict.insert(Value::from("abc"), Value::from("123"));
    /// let value = Value::from(dict);
    /// assert_eq!(value.as_str(), "abc 123");
    /// ```
    fn from(dict: MoltDict) -> Self {
        Value::inner_from_data(DataRep::Dict(Rc::new(dict)))
    }
}

impl From<MoltInt> for Value {
    /// Creates a new `Value` whose data representation is a `MoltInt`.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    ///
    /// let value = Value::from(123);
    /// assert_eq!(value.as_str(), "123");
    /// ```
    fn from(int: MoltInt) -> Self {
        Value::inner_from_data(DataRep::Int(int))
    }
}

impl From<MoltFloat> for Value {
    /// Creates a new `Value` whose data representation is a `MoltFloat`.
    ///
    /// # String Representation
    ///
    /// The string representation of the value will be however Rust's `format!` macro
    /// formats floating point numbers by default.  **Note**: this isn't quite what we
    /// want; Standard TCL goes to great lengths to ensure that the formatted string
    /// will yield exactly the same floating point number when it is parsed.  Rust
    /// will format the number `5.0` as `5`, turning it into a integer if parsed naively. So
    /// there is more work to be done here.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    ///
    /// let value = Value::from(12.34);
    /// assert_eq!(value.as_str(), "12.34");
    /// ```
    fn from(flt: MoltFloat) -> Self {
        Value::inner_from_data(DataRep::Flt(flt))
    }
}

impl From<MoltList> for Value {
    /// Creates a new `Value` whose data representation is a `MoltList`.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    ///
    /// let list = vec![Value::from(1234), Value::from("abc")];
    /// let value = Value::from(list);
    /// assert_eq!(value.as_str(), "1234 abc");
    /// ```
    fn from(list: MoltList) -> Self {
        Value::inner_from_data(DataRep::List(Rc::new(list)))
    }
}

impl From<&[Value]> for Value {
    /// Creates a new `Value` whose data representation is a `MoltList`.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    ///
    /// let values = [Value::from(1234), Value::from("abc")];
    /// let value = Value::from(&values[..]);
    /// assert_eq!(value.as_str(), "1234 abc");
    /// ```
    fn from(list: &[Value]) -> Self {
        Value::inner_from_data(DataRep::List(Rc::new(list.to_vec())))
    }
}

impl Value {
    /// Returns the empty `Value`, a value whose string representation is the empty
    /// string.
    ///
    /// TODO: This should really be a constant, but there's way to build it as one
    /// unless I use lazy_static.
    pub fn empty() -> Value {
        Value::inner_from_string("".into())
    }

    /// Returns the value's string representation as a reference-counted
    /// string.
    ///
    /// **Note**: This is the standard way of retrieving a `Value`'s
    /// string rep, as unlike `to_string` it doesn't create a new `String`.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// let value = Value::from(123);
    /// assert_eq!(value.as_str(), "123");
    /// ```
    pub fn as_str(&self) -> &str {
        // FIRST, get the string rep, computing it from the data_rep if necessary.
        // self.inner.string_rep.get_or_init(|| (self.inner.data_rep.borrow()).to_string())

        // NOTE: This method is the only place where the string_rep is queried.
        let slot = unsafe { &*self.inner.string_rep.get() };

        if let Some(inner) = slot {
            return inner;
        }

        // NOTE: This is the only place where the string_rep is set.
        // Because we returned it if it was Some, it is only ever set once.
        // Thus, this is safe: as_str() is the only way to retrieve the string_rep,
        // and it computes the string_rep lazily after which it is immutable.
        let slot = unsafe { &mut *self.inner.string_rep.get() };
        *slot = Some((self.inner.data_rep.borrow()).to_string());

        slot.as_ref().expect("string rep")
    }

    /// Returns the value's string representation if it is already ready.
    /// This is useful for implementing command line switches.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// let value = Value::from(123);
    /// assert_eq!(value.try_as_str(), None);
    /// assert_eq!(value.as_str(), "123");
    /// assert_eq!(value.try_as_str(), Some("123"));
    /// let value_str = Value::from("123");
    /// assert_eq!(value_str.try_as_str(), Some("123"));
    /// ```
    pub fn try_as_str(&self) -> Option<&str> {
        unsafe { &*self.inner.string_rep.get() }.as_ref().map(|x| x.as_ref())
    }

    /// Tries to return the `Value` as a `bool`, parsing the
    /// value's string representation if necessary.
    ///
    /// # Boolean Strings
    ///
    /// The following are valid boolean strings, regardless of case: `true`,
    /// `false`, `on`, `off`, `yes`, `no`, `1`, `0`.  Note that other numeric strings are
    /// _not_ valid boolean strings.
    ///
    /// # Numeric Values
    ///
    /// Non-negative numbers are interpreted as true; zero is interpreted as false.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// use molt::types::Exception;
    /// # fn dummy() -> Result<bool,Exception> {
    /// // All of the following can be interpreted as booleans.
    /// let value = Value::from(true);
    /// let flag = value.as_bool()?;
    /// assert_eq!(flag, true);
    ///
    /// let value = Value::from("no");
    /// let flag = value.as_bool()?;
    /// assert_eq!(flag, false);
    ///
    /// let value = Value::from(5);
    /// let flag = value.as_bool()?;
    /// assert_eq!(flag, true);
    ///
    /// let value = Value::from(0);
    /// let flag = value.as_bool()?;
    /// assert_eq!(flag, false);
    ///
    /// let value = Value::from(1.1);
    /// let flag = value.as_bool()?;
    /// assert_eq!(flag, true);
    ///
    /// let value = Value::from(0.0);
    /// let flag = value.as_bool()?;
    /// assert_eq!(flag, false);
    ///
    /// // Numeric strings can not, unless evaluated as expressions.
    /// let value = Value::from("123");
    /// assert!(value.as_bool().is_err());
    /// # Ok(true)
    /// # }
    /// ```
    pub fn as_bool(&self) -> Result<bool, Exception> {
        // Extra block, so that the dref is dropped before we borrow mutably.
        {
            let data_ref = self.inner.data_rep.borrow();

            // FIRST, if we have a boolean then just return it.
            if let DataRep::Bool(flag) = *data_ref {
                return Ok(flag);
            }

            // NEXT, if we have a number return whether it's zero or not.
            if let DataRep::Int(int) = *data_ref {
                return Ok(int != 0);
            }

            if let DataRep::Flt(flt) = *data_ref {
                return Ok(flt != 0.0);
            }
        }

        // NEXT, Try to parse the string_rep as a boolean
        let str = self.as_str();
        let flag = Value::get_bool(str)?;
        *(self.inner.data_rep.borrow_mut()) = DataRep::Bool(flag);
        Ok(flag)
    }

    /// Converts a string argument into a boolean, returning an error on failure.
    ///
    /// Molt accepts the following strings as Boolean values:
    ///
    /// * **true**: `true`, `yes`, `on`, `1`
    /// * **false**: `false`, `no`, `off`, `0`
    ///
    /// Parsing is case-insensitive, and leading and trailing whitespace are ignored.
    ///
    /// This method does not evaluate expressions; use `molt::expr` to evaluate boolean
    /// expressions.
    ///
    /// # Example
    ///
    /// ```
    /// # use molt::types::*;
    /// # fn dummy() -> Result<bool,Exception> {
    /// let arg = "yes";
    /// let flag = Value::get_bool(arg)?;
    /// assert!(flag);
    /// # Ok(flag)
    /// # }
    /// ```
    pub fn get_bool(arg: &str) -> Result<bool, Exception> {
        let orig = arg;
        let value: &str = &arg.trim().to_lowercase();
        match value {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            _ => molt_err!("expected boolean but got \"{}\"", orig),
        }
    }

    /// Tries to return the `Value` as an `Rc<MoltDict>`, parsing the
    /// value's string representation if necessary.
    ///
    /// # Example
    ///
    /// ```
    /// use std::rc::Rc;
    /// use molt::types::Value;
    /// use molt::types::MoltDict;
    /// use molt::types::Exception;
    /// # fn dummy() -> Result<(),Exception> {
    ///
    /// let value = Value::from("abc 1234");
    /// let dict: Rc<MoltDict> = value.as_dict()?;
    ///
    /// assert_eq!(dict.len(), 1);
    /// assert_eq!(dict.get(&Value::from("abc")), Some(&Value::from("1234")));
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn as_dict(&self) -> Result<Rc<MoltDict>, Exception> {
        // FIRST, if we have the desired type, return it.
        if let DataRep::Dict(dict) = &*self.inner.data_rep.borrow() {
            return Ok(dict.clone());
        }

        // NEXT, try to parse the string_rep as a list; then turn it into a dict.
        let str = self.as_str();
        let list = get_list(str)?;

        if list.len() % 2 != 0 {
            return molt_err!("missing value to go with key");
        }

        let dict = Rc::new(list_to_dict(&list));

        *self.inner.data_rep.borrow_mut() = DataRep::Dict(dict.clone());

        Ok(dict)
    }

    /// Tries to return the `Value` as a `MoltDict`, parsing the
    /// value's string representation if necessary.
    ///
    /// Use [`as_dict`](#method.as_dict) when simply referring to the dict's content;
    /// use this method when constructing a new dict from the old one.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// use molt::types::MoltDict;
    /// use molt::types::Exception;
    /// # fn dummy() -> Result<String,Exception> {
    ///
    /// let value = Value::from("abc 1234");
    /// let dict: MoltDict = value.to_dict()?;
    /// assert_eq!(dict.len(), 2);
    /// assert_eq!(dict.get(&Value::from("abc")), Some(&Value::from("1234")));
    ///
    /// # Ok("dummy".to_string())
    /// # }
    /// ```
    pub fn to_dict(&self) -> Result<MoltDict, Exception> {
        Ok((&*self.as_dict()?).to_owned())
    }

    /// Tries to return the `Value` as a `MoltInt`, parsing the
    /// value's string representation if necessary.
    ///
    /// # Integer Syntax
    ///
    /// Molt accepts decimal integer strings, and hexadecimal integer strings
    /// with a `0x` prefix.  Strings may begin with a unary "+" or "-".  Hex
    /// digits may be in upper or lower case.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// use molt::types::MoltInt;
    /// use molt::types::Exception;
    /// # fn dummy() -> Result<MoltInt,Exception> {
    ///
    /// let value = Value::from(123);
    /// let int = value.as_int()?;
    /// assert_eq!(int, 123);
    ///
    /// let value = Value::from("OxFF");
    /// let int = value.as_int()?;
    /// assert_eq!(int, 255);
    /// # Ok(1)
    /// # }
    /// ```
    pub fn as_int(&self) -> Result<MoltInt, Exception> {
        // FIRST, if we have an integer then just return it.
        if let DataRep::Int(int) = *self.inner.data_rep.borrow() {
            return Ok(int);
        }

        // NEXT, Try to parse the string_rep as an integer
        let str = self.as_str();
        let int = Value::get_int(str)?;
        *self.inner.data_rep.borrow_mut() = DataRep::Int(int);
        Ok(int)
    }

    /// Converts a string argument into a `MoltInt`, returning an error on failure.
    ///
    /// Molt accepts decimal integer strings, and hexadecimal integer strings
    /// with a `0x` prefix.  Strings may begin with a unary "+" or "-".  Leading and
    /// trailing whitespace is ignored.
    ///
    /// # Example
    ///
    /// ```
    /// # use molt::types::*;
    /// # fn dummy() -> Result<MoltInt,Exception> {
    /// let arg = "1";
    /// let int = Value::get_int(arg)?;
    /// # Ok(int)
    /// # }
    /// ```
    pub fn get_int(arg: &str) -> Result<MoltInt, Exception> {
        let orig = arg;
        let mut arg = arg.trim();
        let mut minus = 1;

        if arg.starts_with('+') {
            arg = &arg[1..];
        } else if arg.starts_with('-') {
            minus = -1;
            arg = &arg[1..];
        }

        let parse_result = if arg.starts_with("0x") {
            MoltInt::from_str_radix(&arg[2..], 16)
        } else {
            arg.parse::<MoltInt>()
        };

        match parse_result {
            Ok(int) => Ok(minus * int),
            Err(_) => molt_err!("expected integer but got \"{}\"", orig),
        }
    }

    /// Tries to return the `Value` as a `MoltFloat`, parsing the
    /// value's string representation if necessary.
    ///
    /// # Floating-Point Syntax
    ///
    /// Molt accepts the same floating-point strings as Rust's standard numeric parser.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// use molt::types::MoltFloat;
    /// use molt::types::Exception;
    /// # fn dummy() -> Result<MoltFloat,Exception> {
    ///
    /// let value = Value::from(12.34);
    /// let flt = value.as_float()?;
    /// assert_eq!(flt, 12.34);
    ///
    /// let value = Value::from("23.45");
    /// let flt = value.as_float()?;
    /// assert_eq!(flt, 23.45);
    /// # Ok(1.0)
    /// # }
    /// ```
    pub fn as_float(&self) -> Result<MoltFloat, Exception> {
        // FIRST, if we have a float then just return it.
        if let DataRep::Flt(flt) = *self.inner.data_rep.borrow() {
            return Ok(flt);
        }

        // NEXT, Try to parse the string_rep as a float
        let str = self.as_str();
        let flt = Value::get_float(str)?;
        *self.inner.data_rep.borrow_mut() = DataRep::Flt(flt);
        Ok(flt)
    }

    /// Converts an string argument into a `MoltFloat`, returning an error on failure.
    ///
    /// Molt accepts any string acceptable to `str::parse<f64>` as a valid floating
    /// point string.  Leading and trailing whitespace is ignored, and parsing is
    /// case-insensitive.
    ///
    /// # Example
    ///
    /// ```
    /// # use molt::types::*;
    /// # fn dummy() -> Result<MoltFloat,Exception> {
    /// let arg = "1e2";
    /// let val = Value::get_float(arg)?;
    /// # Ok(val)
    /// # }
    /// ```
    pub fn get_float(arg: &str) -> Result<MoltFloat, Exception> {
        let arg_trim = arg.trim().to_lowercase();

        match arg_trim.parse::<MoltFloat>() {
            Ok(flt) => Ok(flt),
            Err(_) => molt_err!("expected floating-point number but got \"{}\"", arg),
        }
    }

    /// Computes the string rep for a MoltFloat.
    ///
    /// TODO: This needs a lot of work, so that floating point outputs will parse back into
    /// the same floating point numbers.
    fn fmt_float(f: &mut std::fmt::Formatter, flt: MoltFloat) -> std::fmt::Result {
        if flt == std::f64::INFINITY {
            write!(f, "Inf")
        } else if flt == std::f64::NEG_INFINITY {
            write!(f, "-Inf")
        } else if flt.is_nan() {
            write!(f, "NaN")
        } else {
            // TODO: Needs improvement.
            write!(f, "{}", flt)
        }
    }

    /// Tries to return the `Value` as an `Rc<MoltList>`, parsing the
    /// value's string representation if necessary.
    ///
    /// # Example
    ///
    /// ```
    /// use std::rc::Rc;
    /// use molt::types::Value;
    /// use molt::types::MoltList;
    /// use molt::types::Exception;
    /// # fn dummy() -> Result<String,Exception> {
    ///
    /// let value = Value::from("1234 abc");
    /// let list: Rc<MoltList> = value.as_list()?;
    /// assert_eq!(list.len(), 2);
    ///
    /// assert_eq!(list[0], Value::from("1234"));
    /// assert_eq!(list[1], Value::from("abc"));
    ///
    /// # Ok("dummy".to_string())
    /// # }
    /// ```
    pub fn as_list(&self) -> Result<Rc<MoltList>, Exception> {
        // FIRST, if we have the desired type, return it.
        if let DataRep::List(list) = &*self.inner.data_rep.borrow() {
            return Ok(list.clone());
        }

        // NEXT, try to parse the string_rep as a list.
        let str = self.as_str();
        let list = Rc::new(get_list(str)?);
        *self.inner.data_rep.borrow_mut() = DataRep::List(list.clone());

        Ok(list)
    }

    /// Tries to return the `Value` as a `MoltList`, parsing the
    /// value's string representation if necessary.
    ///
    /// Use [`as_list`](#method.as_list) when simply referring to the list's content;
    /// use this method when constructing a new list from the old one.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// use molt::types::MoltList;
    /// use molt::types::Exception;
    /// # fn dummy() -> Result<String,Exception> {
    ///
    /// let value = Value::from("1234 abc");
    /// let list: MoltList = value.to_list()?;
    /// assert_eq!(list.len(), 2);
    ///
    /// assert_eq!(list[0], Value::from("1234"));
    /// assert_eq!(list[1], Value::from("abc"));
    ///
    /// # Ok("dummy".to_string())
    /// # }
    /// ```
    pub fn to_list(&self) -> Result<MoltList, Exception> {
        Ok((&*self.as_list()?).to_owned())
    }

    /// Tries to return the `Value` as an `Rc<Script>`, parsing the
    /// value's string representation if necessary.
    ///
    /// For internal use only.  Note: this is the normal way to convert a script string
    /// into a Script object.  Converting the Script back into a Tcl string is not
    /// currently supported.
    pub(crate) fn as_script(&self) -> Result<Rc<Script>, Exception> {
        // FIRST, if we have the desired type, return it.
        if let DataRep::Script(script) = &*self.inner.data_rep.borrow() {
            return Ok(script.clone());
        }

        // NEXT, try to parse the string_rep as a script.
        let str = self.as_str();
        let script = Rc::new(parser::parse(str)?);
        *self.inner.data_rep.borrow_mut() = DataRep::Script(script.clone());

        Ok(script)
    }

    /// Returns the `Value` as an `Rc<VarName>`, parsing the
    /// value's string representation if necessary.  This type is usually hidden by the
    /// `Interp`'s `var` and `set_var` methods, which use it implicitly; however it is
    /// available to extension authors if need be.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::{Value, VarName};
    ///
    /// let value = Value::from("my_var");
    /// let var_name = value.as_var_name();
    /// assert_eq!(var_name.name(), "my_var");
    /// assert_eq!(var_name.index(), None);
    ///
    /// let value = Value::from("my_array(1)");
    /// let var_name = value.as_var_name();
    /// assert_eq!(var_name.name(), "my_array");
    /// assert_eq!(var_name.index(), Some("1"));
    /// ```
    pub fn as_var_name(&self) -> Rc<VarName> {
        // FIRST, if we have the desired type, return it.
        if let DataRep::VarName(var_name) = &*self.inner.data_rep.borrow() {
            return var_name.clone();
        }

        // NEXT, try to parse the string_rep as a variable name.
        let var_name = Rc::new(parser::parse_varname_literal(self.as_str()));

        *self.inner.data_rep.borrow_mut() = DataRep::VarName(var_name.clone());
        var_name
    }

    /// Creates a new `Value` containing the given value of some user type.
    ///
    /// The user type must meet certain constraints; see the
    /// [module level documentation](index.html) for details on
    /// how to define an external type for use with Molt.
    ///
    /// # Example
    ///
    /// Suppose we have a type `HexColor` that meets the constraints; we can create
    /// a `Value` containing one as follows.  Notice that the `Value` ownership of its input:
    ///
    /// ```ignore
    /// let color: HexColor::new(0x11u8, 0x22u8, 0x33u8);
    /// let value = Value::from_other(color);
    ///
    /// // Retrieve the value's string rep.
    /// assert_eq!(value.as_str(), "#112233");
    /// ```
    ///
    /// See [`Value::as_other`](#method.as_other) and
    /// [`Value::as_copy`](#method.as_copy) for examples of how to
    /// retrieve a `MyType` value from a `Value`.
    pub fn from_other<T: 'static>(value: T) -> Value
    where
        T: Display + Debug,
    {
        Value::inner_from_data(DataRep::Other(Rc::new(value)))
    }

    /// Tries to interpret the `Value` as a value of external type `T`, parsing
    /// the string representation if necessary.
    ///
    /// The user type must meet certain constraints; see the
    /// [module level documentation](index.html) for details on
    /// how to define an external type for use with Molt.
    ///
    /// # Return Value
    ///
    /// The value is returned as an `Rc<T>`, as this allows the client to
    /// use the value freely and clone it efficiently if needed.
    ///
    /// This method returns `Option<Rc<T>>` rather than `Result<Rc<T>,Exception>`
    /// because it is up to the caller to provide a meaningful error message.
    /// It is normal for externally defined types to wrap this function in a function
    /// that does so; see the [module level documentation](index.html) for an example.
    ///
    /// # Example
    ///
    /// Suppose we have a type `HexColor` that meets the constraints; we can create
    /// a `Value` containing one and retrieve it as follows.
    ///
    /// ```ignore
    /// // Just a normal Molt string
    /// let value = Value::from("#112233");
    ///
    /// // Retrieve it as an Option<Rc<HexColor>>:
    /// let color = value.as_other::<HexColor>()
    ///
    /// if color.is_some() {
    ///     let color = color.unwrap();
    ///     let r = *color.red();
    ///     let g = *color.green();
    ///     let b = *color.blue();
    /// }
    /// ```
    pub fn as_other<T: 'static>(&self) -> Option<Rc<T>>
    where
        T: Display + Debug + FromStr,
    {
        // FIRST, if we have the desired type, return it.
        if let DataRep::Other(other) = &*self.inner.data_rep.borrow() {
            // other is an &Rc<MoltAny>
            if let Ok(out) = other.clone().downcast::<T>() {
                return Some(out);
            }
        }

        // NEXT, can we parse it as a T?  If so, save it back to
        // the data_rep, and return it.
        let str = self.as_str();

        if let Ok(tval) = str.parse::<T>() {
            let tval = Rc::new(tval);
            let out = tval.clone();
            *self.inner.data_rep.borrow_mut() = DataRep::Other(Rc::new(tval));
            return Some(out);
        }

        // NEXT, we couldn't do it.
        None
    }

    /// Tries to interpret the `Value` as a value of type `T`, parsing the string
    /// representation if necessary, and returning a copy.
    ///
    /// The user type must meet certain constraints; and in particular it must
    /// implement `Copy`. See the [module level documentation](index.html) for details on
    /// how to define an external type for use with Molt.
    ///
    /// This method returns `Option` rather than `Result` because it is up
    /// to the caller to provide a meaningful error message.  It is normal
    /// for externally defined types to wrap this function in a function
    /// that does so.
    ///
    /// # Example
    ///
    /// Suppose we have a type `HexColor` that meets the normal external type
    /// constraints and also supports copy; we can create a `Value` containing one and
    /// retrieve it as follows.
    ///
    /// ```ignore
    /// // Just a normal Molt string
    /// let value = Value::from("#112233");
    ///
    /// // Retrieve it as an Option<HexColor>:
    /// let color = value.as_copy::<HexColor>()
    ///
    /// if color.is_some() {
    ///     let color = color.unwrap();
    ///     let r = color.red();
    ///     let g = color.green();
    ///     let b = color.blue();
    /// }
    /// ```
    pub fn as_copy<T: 'static>(&self) -> Option<T>
    where
        T: Display + Debug + FromStr + Copy,
    {
        // FIRST, if we have the desired type, return it.
        if let DataRep::Other(other) = &*self.inner.data_rep.borrow() {
            // other is an &Rc<MoltAny>
            if let Ok(out) = other.clone().downcast::<T>() {
                return Some(*out);
            }
        }

        // NEXT, can we parse it as a T?  If so, save it back to
        // the data_rep, and return it.
        let str = self.as_str();

        if let Ok(tval) = str.parse::<T>() {
            let tval = Rc::new(tval);
            let out = tval.clone();
            *self.inner.data_rep.borrow_mut() = DataRep::Other(Rc::new(tval));
            return Some(*out);
        }

        // NEXT, we couldn't do it.
        None
    }

    /// For use by `expr::expr` in parsing out `Values`.
    pub(crate) fn already_number(&self) -> Option<Datum> {
        let iref = self.inner.data_rep.borrow();

        match *iref {
            DataRep::Flt(flt) => Some(Datum::float(flt)),
            DataRep::Int(int) => Some(Datum::int(int)),
            _ => None,
        }
    }
}

//-----------------------------------------------------------------------------
// The MoltAny Trait: a tool for handling external types.

/// This trait allows us to except "other" types, and still compute their
/// string rep on demand.
trait MoltAny: Any + Display + Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl dyn MoltAny {
    /// Is this value a value of the desired type?
    pub fn is<T: 'static>(&self) -> bool {
        TypeId::of::<T>() == self.type_id()
    }

    /// Downcast an `Rc<MoltAny>` to an `Rc<T>`
    fn downcast<T: 'static>(self: Rc<Self>) -> Result<Rc<T>, Rc<Self>> {
        if self.is::<T>() {
            unsafe { Ok(Rc::from_raw(Rc::into_raw(self) as _)) }
        } else {
            Err(self)
        }
    }
}

impl<T: Any + Display + Debug> MoltAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

//-----------------------------------------------------------------------------
// DataRep enum: a sum type for the different kinds of data_reps.

// The data representation for Values.
#[derive(Clone, Debug)]
enum DataRep {
    /// A Boolean
    Bool(bool),

    /// A Molt Dictionary
    Dict(Rc<MoltDict>),

    /// A Molt integer
    Int(MoltInt),

    /// A Molt float
    Flt(MoltFloat),

    /// A Molt List
    List(Rc<MoltList>),

    /// A Script
    Script(Rc<Script>),

    /// A Variable Name
    VarName(Rc<VarName>),

    /// An external data type
    Other(Rc<dyn MoltAny>),

    /// The Value has no data rep at present.
    None,
}

impl Display for DataRep {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DataRep::Bool(flag) => write!(f, "{}", if *flag { 1 } else { 0 }),
            DataRep::Dict(dict) => write!(f, "{}", dict_to_string(&*dict)),
            DataRep::Int(int) => write!(f, "{}", int),
            DataRep::Flt(flt) => Value::fmt_float(f, *flt),
            DataRep::List(list) => write!(f, "{}", list_to_string(&*list)),
            DataRep::Script(script) => write!(f, "{:?}", script),
            DataRep::VarName(var_name) => write!(f, "{:?}", var_name),
            DataRep::Other(other) => write!(f, "{}", other),
            DataRep::None => write!(f, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dict::dict_new;
    use std::fmt;
    use std::str::FromStr;

    #[test]
    fn from_string() {
        // Using From<String>
        let val = Value::from("xyz".to_string());
        assert_eq!(&val.to_string(), "xyz");

        // Using Into
        let val: Value = String::from("Fred").into();
        assert_eq!(&val.to_string(), "Fred");
    }

    #[test]
    fn from_str_ref() {
        // Using From<&str>
        let val = Value::from("xyz");
        assert_eq!(&val.to_string(), "xyz");

        // Using Into
        let val: Value = "Fred".into();
        assert_eq!(&val.to_string(), "Fred");
    }

    #[test]
    fn clone_string() {
        // Values with just string reps can be cloned and have equal string reps.
        let val = Value::from("abc");
        let val2 = val.clone();
        assert_eq!(*val.to_string(), *val2.to_string());
    }

    #[test]
    fn as_str() {
        let val = Value::from("abc");
        assert_eq!(val.as_str(), "abc");

        let val2 = val.clone();
        assert_eq!(val.as_str(), val2.as_str());
    }

    #[test]
    fn compare() {
        let val = Value::from("123");
        let val2 = Value::from(123);
        let val3 = Value::from(456);

        assert_eq!(val, val2);
        assert_ne!(val, val3);
    }

    #[test]
    fn from_bool() {
        // Using From<bool>
        let val = Value::from(true);
        assert_eq!(&val.to_string(), "1");

        let val = Value::from(false);
        assert_eq!(&val.to_string(), "0");
    }

    #[test]
    fn as_bool() {
        // Can convert string to bool.
        let val = Value::from("true");
        assert_eq!(val.as_bool(), Ok(true));

        // Non-zero numbers are true; zero is false.
        let val = Value::from(5);
        assert_eq!(val.as_bool(), Ok(true));

        let val = Value::from(0);
        assert_eq!(val.as_bool(), Ok(false));

        let val = Value::from(5.5);
        assert_eq!(val.as_bool(), Ok(true));

        let val = Value::from(0.0);
        assert_eq!(val.as_bool(), Ok(false));
    }

    #[test]
    fn get_bool() {
        // Test the underlying boolean value parser.
        assert_eq!(Ok(true), Value::get_bool("1"));
        assert_eq!(Ok(true), Value::get_bool("true"));
        assert_eq!(Ok(true), Value::get_bool("yes"));
        assert_eq!(Ok(true), Value::get_bool("on"));
        assert_eq!(Ok(true), Value::get_bool("TRUE"));
        assert_eq!(Ok(true), Value::get_bool("YES"));
        assert_eq!(Ok(true), Value::get_bool("ON"));
        assert_eq!(Ok(false), Value::get_bool("0"));
        assert_eq!(Ok(false), Value::get_bool("false"));
        assert_eq!(Ok(false), Value::get_bool("no"));
        assert_eq!(Ok(false), Value::get_bool("off"));
        assert_eq!(Ok(false), Value::get_bool("FALSE"));
        assert_eq!(Ok(false), Value::get_bool("NO"));
        assert_eq!(Ok(false), Value::get_bool("OFF"));
        assert_eq!(Ok(true), Value::get_bool(" true "));
        assert_eq!(
            Value::get_bool("nonesuch"),
            molt_err!("expected boolean but got \"nonesuch\"")
        );
        assert_eq!(
            Value::get_bool(" Nonesuch "),
            molt_err!("expected boolean but got \" Nonesuch \"")
        );
    }

    #[test]
    fn from_as_dict() {
        // NOTE: we aren't testing dict formatting and parsing here.
        // We *are* testing that Value can convert dicts to and from strings.
        // and back again.
        let mut dict = dict_new();
        dict.insert(Value::from("abc"), Value::from("def"));
        let dictval = Value::from(dict);
        assert_eq!(dictval.as_str(), "abc def");

        let dictval = Value::from("qrs xyz");
        let result = dictval.as_dict();

        assert!(result.is_ok());

        if let Ok(rcdict) = result {
            assert_eq!(rcdict.len(), 1);
            assert_eq!(rcdict.get(&Value::from("qrs")), Some(&Value::from("xyz")));
        }
    }

    #[test]
    fn to_dict() {
        let dictval = Value::from("qrs xyz");
        let result = dictval.to_dict();

        assert!(result.is_ok());

        if let Ok(dict) = result {
            assert_eq!(dict.len(), 1);
            assert_eq!(dict.get(&Value::from("qrs")), Some(&Value::from("xyz")));
        }
    }

    #[test]
    fn from_as_int() {
        let val = Value::from(5);
        assert_eq!(val.as_str(), "5");
        assert_eq!(val.as_int(), Ok(5));
        assert_eq!(val.as_float(), Ok(5.0));

        let val = Value::from("7");
        assert_eq!(val.as_str(), "7");
        assert_eq!(val.as_int(), Ok(7));
        assert_eq!(val.as_float(), Ok(7.0));

        // TODO: Note, 7.0 might not get converted to "7" long term.
        // In Standard TCL, its string_rep would be "7.0".  Need to address
        // MoltFloat formatting/parsing.
        let val = Value::from(7.0);
        assert_eq!(val.as_str(), "7");
        assert_eq!(val.as_int(), Ok(7));
        assert_eq!(val.as_float(), Ok(7.0));

        let val = Value::from("abc");
        assert_eq!(val.as_int(), molt_err!("expected integer but got \"abc\""));
    }

    #[test]
    fn get_int() {
        // Test the internal integer parser
        assert_eq!(Value::get_int("1"), Ok(1));
        assert_eq!(Value::get_int("-1"), Ok(-1));
        assert_eq!(Value::get_int("+1"), Ok(1));
        assert_eq!(Value::get_int("0xFF"), Ok(255));
        assert_eq!(Value::get_int("+0xFF"), Ok(255));
        assert_eq!(Value::get_int("-0xFF"), Ok(-255));
        assert_eq!(Value::get_int(" 1 "), Ok(1));

        assert_eq!(
            Value::get_int(""),
            molt_err!("expected integer but got \"\"")
        );
        assert_eq!(
            Value::get_int("a"),
            molt_err!("expected integer but got \"a\"")
        );
        assert_eq!(
            Value::get_int("0x"),
            molt_err!("expected integer but got \"0x\"")
        );
        assert_eq!(
            Value::get_int("0xABGG"),
            molt_err!("expected integer but got \"0xABGG\"")
        );
        assert_eq!(
            Value::get_int(" abc "),
            molt_err!("expected integer but got \" abc \"")
        );
    }

    #[test]
    fn from_as_float() {
        let val = Value::from(12.5);
        assert_eq!(val.as_str(), "12.5");
        assert_eq!(val.as_int(), molt_err!("expected integer but got \"12.5\""));
        assert_eq!(val.as_float(), Ok(12.5));

        let val = Value::from("7.8");
        assert_eq!(val.as_str(), "7.8");
        assert_eq!(val.as_int(), molt_err!("expected integer but got \"7.8\""));
        assert_eq!(val.as_float(), Ok(7.8));

        // TODO: Problem here: tries to mutably borrow the data_rep to convert from int to string
        // while the data_rep is already mutably borrowed to convert the string to float.
        let val = Value::from(5);
        assert_eq!(val.as_float(), Ok(5.0));

        let val = Value::from("abc");
        assert_eq!(
            val.as_float(),
            molt_err!("expected floating-point number but got \"abc\"")
        );
    }

    #[test]
    fn get_float() {
        // Test the internal float parser.
        // NOTE: At present, it relies on the standard Rust float parser, so only
        // check special case behavior.
        assert_eq!(Value::get_float("1"), Ok(1.0));
        assert_eq!(Value::get_float("2.3"), Ok(2.3));
        assert_eq!(Value::get_float(" 4.5 "), Ok(4.5));
        assert_eq!(Value::get_float("Inf"), Ok(std::f64::INFINITY));

        assert_eq!(
            Value::get_float("abc"),
            molt_err!("expected floating-point number but got \"abc\"")
        );
        assert_eq!(
            Value::get_float(" abc "),
            molt_err!("expected floating-point number but got \" abc \"")
        );
    }

    #[test]
    fn from_as_list() {
        // NOTE: we aren't testing list formatting and parsing here; that's done in list.rs.
        // We *are* testing that Value will use the list.rs code to convert strings to lists
        // and back again.
        let listval = Value::from(vec![Value::from("abc"), Value::from("def")]);
        assert_eq!(listval.as_str(), "abc def");

        let listval = Value::from("qrs xyz");
        let result = listval.as_list();

        assert!(result.is_ok());

        if let Ok(rclist) = result {
            assert_eq!(rclist.len(), 2);
            assert_eq!(rclist[0].to_string(), "qrs".to_string());
            assert_eq!(rclist[1].to_string(), "xyz".to_string());
        }
    }

    #[test]
    fn to_list() {
        let listval = Value::from(vec![Value::from("abc"), Value::from("def")]);
        let result = listval.to_list();

        assert!(result.is_ok());
        let list: MoltList = result.expect("an owned list");

        assert_eq!(list.len(), 2);
        assert_eq!(list[0].to_string(), "abc".to_string());
        assert_eq!(list[1].to_string(), "def".to_string());
    }

    #[test]
    fn as_script() {
        let val = Value::from("a");
        assert!(val.as_script().is_ok());

        let val = Value::from("a {b");
        assert_eq!(val.as_script(), molt_err!("missing close-brace"));
    }

    #[test]
    fn as_var_name() {
        let val = Value::from("a");
        assert_eq!(val.as_var_name().name(), "a");
        assert_eq!(val.as_var_name().index(), None);

        let val = Value::from("a(b)");
        assert_eq!(val.as_var_name().name(), "a");
        assert_eq!(val.as_var_name().index(), Some("b"));
    }

    #[test]
    fn from_value_slice() {
        // NOTE: we aren't testing list formatting and parsing here; that's done in list.rs.
        // We *are* testing that Value will use the list.rs code to convert strings to lists
        // and back again.
        let array = [Value::from("abc"), Value::from("def")];
        let listval = Value::from(&array[..]);
        assert_eq!(listval.as_str(), "abc def");
    }

    #[test]
    fn from_to_flavor() {
        // Give a Flavor, get an Rc<Flavor> back.
        let myval = Value::from_other(Flavor::SALTY);
        let result = myval.as_other::<Flavor>();
        assert!(result.is_some());
        let out = result.unwrap();
        assert_eq!(*out, Flavor::SALTY);

        // Give a String, get an Rc<Flavor> back.
        let myval = Value::from("sweet");
        let result = myval.as_other::<Flavor>();
        assert!(result.is_some());
        let out = result.unwrap();
        assert_eq!(*out, Flavor::SWEET);

        // Flavor is Copy, so get a Flavor back
        let myval = Value::from_other(Flavor::SALTY);
        let result = myval.as_copy::<Flavor>();
        assert!(result.is_some());
        let out = result.unwrap();
        assert_eq!(out, Flavor::SALTY);
    }

    #[test]
    fn already_number() {
        // Can retrieve a DataRep::Int as a Datum::Int.
        let value = Value::from(123);
        let out = value.already_number();
        assert!(out.is_some());
        assert_eq!(out.unwrap(), Datum::int(123));

        // Can retrieve a DataRep::Flt as a Datum::Flt.
        let value = Value::from(45.6);
        let out = value.already_number();
        assert!(out.is_some());
        assert_eq!(out.unwrap(), Datum::float(45.6));

        // Other values, None.
        let value = Value::from("123");
        assert!(value.already_number().is_none());
    }

    // Sample external type, used for testing.

    #[derive(Debug, PartialEq, Copy, Clone)]
    pub enum Flavor {
        SALTY,
        SWEET,
    }

    impl FromStr for Flavor {
        type Err = String;

        fn from_str(value: &str) -> Result<Self, Self::Err> {
            let value = value.to_lowercase();

            if value == "salty" {
                Ok(Flavor::SALTY)
            } else if value == "sweet" {
                Ok(Flavor::SWEET)
            } else {
                Err("Not a flavor string".to_string())
            }
        }
    }

    impl fmt::Display for Flavor {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if *self == Flavor::SALTY {
                write!(f, "salty")
            } else {
                write!(f, "sweet")
            }
        }
    }
}
