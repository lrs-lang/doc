// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// Print an error to stderr and exit.
macro_rules! errexit {
    ($fmt:expr) => { errexit!(concat!($fmt, "{}"), "") };
    ($fmt:expr, $($arg:tt)*) => {{
        errln!($fmt, $($arg)*);
        process::exit(1);
    }};
}

macro_rules! tryerr {
    ($val:expr, $fmt:expr) => { tryerr!($val, concat!($fmt, "{}"), "") };
    ($val:expr, $fmt:expr, $($arg:tt)*) => {{
        match $val {
            Ok(x) => x,
            Err(e) => errexit!(concat!("lrs_doc: ", $fmt, ": ({:?})"), $($arg)*, e),
        }
    }};
}
