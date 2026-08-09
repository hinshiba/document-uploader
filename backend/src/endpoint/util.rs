macro_rules! return_500_with_log {
    ($err:expr) => {
        {
            ::tracing::error!("{}", $err);

            ( ::axum::http::StatusCode::INTERNAL_SERVER_ERROR
            , ::std::result::Result::Err(crate::endpoint::EndpointError {
                message: "unexpected error occured".to_owned(),
                details: ::std::option::Option::None,
            })
            )
        }
    };
}
