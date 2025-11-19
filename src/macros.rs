#[macro_export]
macro_rules! handler {
    ($fn_name:ident) => {
        $crate::paste::paste! {
            pub struct [<$fn_name:camel Handler>];

            #[async_trait::async_trait]
            impl $crate::types::Handler for [<$fn_name:camel Handler>] {
                async fn run(
                    &self,
                    req: &mut $crate::request::RequestBody,
                    res: &mut $crate::response::ResponseWriter
                ) {
                    $fn_name(req, res).await;
                }
            }
            #[allow(non_upper_case_globals)]
            pub const [<$fn_name:camel>]: [<$fn_name:camel Handler>] = [<$fn_name:camel Handler>];

        }
    };
}

#[macro_export]
macro_rules! middleware {
    ($fn_name:ident) => {
        $crate::paste::paste! {
            pub struct [<$fn_name:camel Middleware>];

            #[async_trait::async_trait]
            impl $crate::types::Middleware for [<$fn_name:camel Middleware>] {
                async fn run(&self, req: &mut $crate::request::RequestBody, res: &mut $crate::response::ResponseWriter) {
                    $fn_name(req, res).await;
                }
            }

            #[allow(dead_code, non_upper_case_globals)]
            pub const [<$fn_name:camel>]: [<$fn_name:camel Middleware>] = [<$fn_name:camel Middleware>];
        }
    };
}

#[macro_export]
macro_rules! error {
    ($fn_name:ident) => {
        $crate::paste::paste! {
            pub struct [<$fn_name:camel ErrorHandler>];

            #[async_trait::async_trait]
            impl $crate::types::ErrorHandler for [<$fn_name:camel ErrorHandler>] {
                async fn run(&self, msg: String, res: &mut $crate::response::ResponseWriter) {
                    $fn_name(msg,res).await;
                }
            }

            #[allow(non_upper_case_globals, dead_code)]
            pub const [<$fn_name:camel>]: [<$fn_name:camel ErrorHandler>] = [<$fn_name:camel ErrorHandler>];
        }
    };
}

#[macro_export]
macro_rules! Get {
    ($app:ident, $path:expr, $fn_name:ident) => {
        $crate::paste::paste! {
            $crate::handler!($fn_name);
        $app.get($path, [<$fn_name:camel>]);
        }
    };
}

#[macro_export]
macro_rules! Post {
    ($app:ident, $path:expr, $fn_name:ident) => {
        $crate::paste::paste! {
            $crate::handler!($fn_name);
        $app.post($path, [<$fn_name:camel>]);
        }
    };
}

#[macro_export]
macro_rules! Patch {
    ($app:ident, $path:expr, $fn_name:ident) => {
        $crate::paste::paste! {
            $crate::handler!($fn_name);
        $app.patch($path, [<$fn_name:camel>]);
        }
    };
}

#[macro_export]
macro_rules! Put {
    ($app:ident, $path:expr, $fn_name:ident) => {
        $crate::paste::paste! {
            $crate::handler!($fn_name);
        $app.put($path, [<$fn_name:camel>]);
        }
    };
}

#[macro_export]
macro_rules! Delete {
    ($app:ident, $path:expr, $fn_name:ident) => {
        $crate::paste::paste! {
            $crate::handler!($fn_name);
        $app.delete($path, [<$fn_name:camel>]);
        }
    };
}

#[macro_export]
macro_rules! Middleware {
    ($app:ident, $path:expr, $method:ident ,$fn_name:ident) => {
        $crate::paste::paste! {
            $crate::middleware!($fn_name);

            $app.middleware($path, Some($crate::types::Method::$method) , [<$fn_name:camel>]);
        }
    };

    ($app:ident, $path:expr ,$fn_name:ident) => {
        $crate::paste::paste! {
            $crate::middleware!($fn_name);
            $app.middleware($path, None , [<$fn_name:camel>]);
        }
    };
}

#[macro_export]
macro_rules! Error {
    ($app:ident,$fn_name:ident) => {
        $crate::paste::paste! {
            $crate::error!($fn_name);

            $app.set_error_handler([<$fn_name:camel>]);
        }
    };
}
