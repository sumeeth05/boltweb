#[macro_export]
macro_rules! bolt_handler {
    ($fn_name:ident) => {
        $crate::paste::paste! {
            pub struct [<$fn_name:camel Handler>];

            #[ $crate::async_trait::async_trait ]
            impl $crate::types::Handler for [<$fn_name:camel Handler>] {
                async fn handle(
                    &self,
                    req: &mut $crate::request::RequestBody,
                    res: &mut $crate::response::ResponseWriter
                ) {
                    $fn_name(req, res).await;
                }
            }

            #[allow(non_upper_case_globals)]
            pub const [<$fn_name:upper _HANDLER>]: [<$fn_name:camel Handler>] = [<$fn_name:camel Handler>];
        }
    };
}

#[macro_export]
macro_rules! bolt_middleware {
    ($fn_name:ident) => {
        $crate::paste::paste! {
            pub struct [<$fn_name:camel Middleware>];

            #[ $crate::async_trait::async_trait ]
            impl $crate::types::Middleware for [<$fn_name:camel Middleware>] {
                async fn run(&self, req: &mut $crate::request::RequestBody, res: &mut $crate::response::ResponseWriter) {
                    $fn_name(req, res).await;
                }
            }

            #[allow(dead_code, non_upper_case_globals)]
            pub const [<$fn_name:upper _MIDDLEWARE>]: [<$fn_name:camel Middleware>] = [<$fn_name:camel Middleware>];
        }
    };
}

#[macro_export]
macro_rules! bolt_error_handler {
    ($fn_name:ident) => {
        $crate::paste::paste! {
            pub struct [<$fn_name:camel ErrorHandler>];

            #[ $crate::async_trait::async_trait ]
            impl $crate::types::ErrorHandler for [<$fn_name:camel ErrorHandler>] {
                async fn run(&self, msg: String, res: &mut $crate::response::ResponseWriter) {
                    $fn_name(msg, res).await;
                }
            }

            #[allow(non_upper_case_globals, dead_code)]
            pub const [<$fn_name:upper _ERROR_HANDLER>]: [<$fn_name:camel ErrorHandler>] = [<$fn_name:camel ErrorHandler>];
        }
    };
}
