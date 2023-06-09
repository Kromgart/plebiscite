#[macro_export]
macro_rules! pg_fn_option {
    ($db:ident, $fn:literal, $args:tt)  => {
        pg_fn_option!($db, $fn, $args, ())
    };

    ($db:ident, $fn:literal, [$($args:expr),*], $($output:tt)+)  => {
        {
            const QUERY: &'static str = make_pg_fn_query!($fn, [$($args);*], $($output)+);
            println!("{}", QUERY);

            $db.query_opt(QUERY, &[$($args),*])
                .await
                .map(|opt| opt.map(|row| {
                    //trace_macros!(true);
                    let r = init_from_row!(row, $($output)+);
                    //trace_macros!(false);
                    r
                }))
        }
    };
}

#[macro_export]
macro_rules! pg_fn_vector {
    ($db:ident, $fn:literal, $args:tt)  => {
        pg_fn_vector!($db, $fn, $args, ())
    };

    ($db:ident, $fn:literal, [$($args:expr),*], $($output:tt)+)  => {
        {
            const QUERY: &'static str = make_pg_fn_query!($fn, [$($args);*], $($output)+);
            //println!("{}", QUERY);

            $db.query_vector(QUERY, &[$($args),*])
                .await
                .map(|v| v.into_iter().map(|row| {
                    init_from_row!(row, $($output)+)
                }).collect())
        }
    };
}

//--------------------------------------------------------------------------

macro_rules! make_pg_fn_query {
    ($fn:literal, [$($args:tt)*], $($output:tt)+)  => {
        const_format::formatcp!(
            "SELECT {} {}({});",
            make_columns!($($output)+),
            $fn,
            args_list!(1u8, $($args)*)
        )
    };
}

macro_rules! init_from_row {
    ($row:ident, ()) => {
        $row.get(0)
    };

    ($row:ident, $($tokens:tt)+) => {
        extract_value!({}, 0usize, $row, [$($tokens)+])
    };
}

//--------------------------------------------------------------------------

macro_rules! args_list {

    ($idx:expr,) => { "" };

    ($idx:expr, $arg:expr) => {
        const_format::concatcp!("$", $idx)
    };

    ($idx:expr, $arg:expr; $($tail:tt)+) => {
        const_format::concatcp!("$", $idx, ", ", args_list!(($idx + 1u8), $($tail)+))
    };
}

//--------------------------------------------------------------------------

macro_rules! make_columns_tuple {

    ( { $cont_fn:ident, $cont_inner:tt, $cont_tail:tt }, $acc:tt, []) => {{
        $cont_fn!($cont_inner, $acc, $cont_tail)
    }};

    // ------- subtuples

    ($cont:tt, $acc:tt, [($($tuple:tt)+) $(, $($tail:tt)+)?]) => {
        make_columns_tuple!(
            { make_columns_tuple, $cont, [$($($tail)+)?] },
            $acc,
            [$($tuple)+]
        )
    };

    // ------ into struct

    ($cont:tt, $acc:tt, [$struct:ty { $($flds:tt)+ } $(, $($tail:tt)+ )?]) => {
        make_columns_struct!(
            { make_columns_tuple, $cont, [$($($tail)+)?] },
            $acc,
            [$($flds)+])
    };

    // -------- scalar values

    ( { $cont_fn:ident, $cont_inner:tt, $cont_tail:tt }, [$($acc:tt)*], [$($column:literal)?] ) => {
        $cont_fn!($cont_inner, [$($acc)* $($column)?], $cont_tail)
    };

    ( $cont:tt, [$($acc:tt)*], [$column:literal, $($tail:tt)+] ) => {
        make_columns_tuple!($cont, [$($acc)* $column], [$($tail)+])
    };
}

macro_rules! make_columns_struct {

    ( { $cont_fn:ident, $cont_inner:tt, $cont_tail:tt }, $acc:tt, []) => {{
        $cont_fn!($cont_inner, $acc, $cont_tail)
    }};

    // ------- into tuples

    ($cont:tt, $acc:tt, [$_fld:ident: ($($tuple:tt)+) $(, $($tail:tt)+)?]) => {
        make_columns_tuple!(
            { make_columns_struct, $cont, [$($($tail)+)?] },
            $acc,
            [$($tuple)+]
        )
    };

    // ------ Substruct fields

    ($cont:tt, $acc:tt, [$_fld:ident: $struct:ty { $($flds:tt)+ } $(, $($tail:tt)+ )?]) => {
        make_columns_struct!(
            { make_columns_struct, $cont, [$($($tail)+)?] },
            $acc,
            [$($flds)+])
    };

    // ------ Scalar fields (+ column remaps)

    ( { $cont_fn:ident, $cont_inner:tt, $cont_tail:tt }, [$($acc:tt)*], [$fld:ident] ) => {
        $cont_fn!($cont_inner, [$($acc)* $fld], $cont_tail)
    };

    ( { $cont_fn:ident, $cont_inner:tt, $cont_tail:tt }, [$($acc:tt)*], [$_fld:ident<$column:literal>] ) => {
        $cont_fn!($cont_inner, [$($acc)* $column], $cont_tail)
    };

    ( $cont:tt, [$($acc:tt)*], [$fld:ident, $($tail:tt)+ ]) => {
        make_columns_struct!($cont, [$($acc)* $fld], [$($tail)+])
    };

    ( $cont:tt, [$($acc:tt)*], [$_fld:ident<$column:literal>, $($tail:tt)+ ]) => {
        make_columns_struct!($cont, [$($acc)* $column], [$($tail)+])
    };

}

macro_rules! print_columns {
    ( $_cont:tt, [$($acc:tt)+], $_tail:tt) => {{
        stringify!($($acc),+ FROM)
    }};
}

macro_rules! make_columns {
    ( () ) => { "" };

    ( ($($tuple:tt)+) ) => {
        make_columns_tuple!( { print_columns, {}, [] }, [], [$($tuple)+] )
    };

    ($struct:ident { $($fields:tt)+ }) => {
        make_columns_struct!( { print_columns, {}, [] }, [], [$($fields)+] )
    };
}

//--------------------------------------------------------------------------

macro_rules! extract_value {
    ({}, $idx:expr, $row:ident, [$substruct:ident { $($subflds:tt)+ } $($tail:tt)*]) => {
        init_struct!({}, $idx, $row, [], [$($subflds)+], $substruct)
    };

    ( { $cont_fn:ident, $cont_inner:tt, $cont_acc:tt, $cont_context:tt },
        $idx:expr, $row:ident, [$substruct:ident { $($subflds:tt)+ } $($tail:tt)*]) => {
        init_struct!(
            { $cont_fn, $cont_inner, $cont_acc, [$($tail)*], $cont_context },
            $idx, $row, [], [$($subflds)+], $substruct
        )
    };

    ({}, $idx:expr, $row:ident, [($($subtuple:tt)+) $($tail:tt)*]) => {
        init_tuple!({}, $idx, $row, [], [$($subtuple)+])
    };

    ( { $cont_fn:ident, $cont_inner:tt, $cont_acc:tt, $cont_context:tt },
        $idx:expr, $row:ident, [($($subtuple:tt)+) $($tail:tt)*]) => {
        init_tuple!(
            { $cont_fn, $cont_inner, $cont_acc, [$($tail)*], $cont_context },
            $idx, $row, [], [$($subtuple)+]
        )
    };
}

macro_rules! init_tuple {

    // ----------- top level tuple finished -------------

    //({}, $idx_:tt, $_row:tt, [ $($val:tt)+ ], []) => {
    //    ( $($val),+ )
    ({}, $idx_:tt, $_row:tt, [ $( ( $($val:tt)+ ) )+ ], []) => {
        ( $( $($val)+ ),+ )
    };

    // ----------------  tail is empty ------------------
    // construct this value and send it to the owner's continuation

    ( { $cont_fn:ident, $cont_inner:tt, $cont_acc:tt, $cont_tail:tt, $cont_context:tt },
        $idx:expr, $row:ident, [$(($($val:tt)+))+], []) => {

        $cont_fn!(@add_value 
            { value: ($($($val)+),+), context: $cont_context },
            $cont_inner, $idx, $row, $cont_acc, $cont_tail
        ) 
    };

    // -------- add scalar value to this tuple ----------

    ($cont:tt, $idx:expr, $row:ident, [$($acc:tt)*], [$_key:literal $(, $($tail:tt)+)?]) => {
        init_tuple!($cont, $idx + 1, $row, [$($acc)* ($row.get($idx))], [ $($($tail)+)? ] )
    };

    // ---------- nested non-scalar value ---------------

    ($cont:tt, $idx:expr, $row:ident, $acc:tt, [$($tail:tt)+]) => {
        extract_value!(
            { init_tuple, $cont, $acc, {} },
            $idx, $row, [$($tail)+]
        )
    };

    // ---------- add subvalue and continue -------------

    (@add_value { value: $value:tt, context: {} }, $cont:tt, $idx:expr, $row:ident, [$($acc:tt)*], $tail:tt ) => {
        init_tuple!($cont, $idx, $row, [$($acc)* ($value)], $tail)
    };
}

macro_rules! init_struct {

    // ----------- top-level struct

    ( {}, $idx:expr, $row:ident, [$(($fld:ident, $val:expr))+], [], $struct:ident ) => {
        $struct { $($fld: $val),+ }
    };

    // ----------- finish this value and add it into the owner object

    ( { $cont_fn:ident, $cont_inner:tt, $cont_acc:tt, $cont_tail:tt, $cont_context:tt },
        $idx:expr, $row:ident,
        [$(($fld:ident, $val:expr))+], [], $struct:ident ) => {

        $cont_fn!(@add_value { value: ($struct { $($fld: $val),+ }), context: $cont_context },
            $cont_inner, $idx, $row, $cont_acc, $cont_tail)
    };

    // ---------- scalar value

    ($cont:tt, $idx:expr, $row:ident, [$($acc:tt)*], [$fld:ident$(<$_key:literal>)? $(, $($tail:tt)+)?], $struct:ident) => {
        init_struct!($cont, $idx + 1, $row, [$($acc)* ($fld, $row.get($idx))], [ $($($tail)+)? ], $struct)
    };

    // ---------- extracting nested object

    ($cont:tt, $idx:expr, $row:ident, $acc:tt, [$target_fld:ident: $($tail:tt)+], $struct:ident) => {
        extract_value!(
            { init_struct, $cont, $acc, { $struct, $target_fld } },
            $idx, $row, [$($tail)+]
        )
    };

    // ---------- add nested object and continue

    (@add_value { value: ($($value:tt)+), context: { $cont_struct:ident, $target_fld:ident} },
        $cont:tt, $idx:expr, $row:ident, [$($acc:tt)*], $tail:tt) => {
        init_struct!($cont, $idx, $row, [$($acc)* ($target_fld, $($value)+)], $tail, $cont_struct)
    };
}
